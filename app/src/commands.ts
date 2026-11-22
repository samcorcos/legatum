// Player commands shared by the map, the action bar and keyboard shortcuts.
import { atWar, peacefulAt, peacefulRangedAt, queueDiplomacy, warDeclaration, warQueuedNow } from './diplomacy';
import { isEliminated, useGame } from './store';
import type { Hex, Order, PlayerView, TeamId, UnitView } from './types';
import { clearUnitOrder, pendingUnitOrder, setAirOrder, setUnitOrder, unitsNeedingOrders } from './orders';
import { distance, hexEq } from './hex';
import { shotBlockReason } from './los';
import { feel } from './feel';
import { attacksInMelee, canAutoExplore } from './data';
import { previewPath } from './path';
import { isExploring } from './automation';
import { foundCityUnitBlocker } from './settlement';
import { answerDecision } from './notice';
import { empireTitle } from './empires';
import { occupationRefusal } from './ui/forecastWarnings';

const st = () => useGame.getState();

function seatView(): PlayerView | null {
  const s = st();
  return s.views ? s.views[s.seat] : null;
}

export function selectedOwnUnit(): UnitView | null {
  const s = st();
  const v = seatView();
  if (!v || v.finished || isEliminated(v) || s.selection?.kind !== 'unit') return null;
  const id = s.selection.id;
  const u = v.units.find((x) => x.id === id);
  return u && u.owner === v.me ? u : null;
}

/** Enemy (other team) unit or city at a hex, if any is known. */
export function enemyAt(v: PlayerView, h: Hex): { kind: 'unit' | 'city'; owner: number } | null {
  const c = v.cities.find((c) => hexEq(c.pos, h) && atWar(v, c.owner));
  if (c) return { kind: 'city', owner: c.owner };
  const u = v.units.find((u) => hexEq(u.pos, h) && atWar(v, u.owner));
  if (u) return { kind: 'unit', owner: u.owner };
  return null;
}

export { peacefulAt, peacefulRangedAt, warDeclaration, warQueuedNow };

/** A target a ranged attack can hit now: a military unit or city of a team at War (or barbarians). */
export function rangedEnemyAt(v: PlayerView, h: Hex): boolean {
  return v.cities.some((c) => hexEq(c.pos, h) && atWar(v, c.owner))
    || v.units.some((u) => hexEq(u.pos, h) && u.military && atWar(v, u.owner));
}

/** Ask before attacking a team at Peace (Civ-style). `unit` null: declaring from a city card. */
export function promptWar(team: TeamId, unit: number | null = null, target: Hex | null = null, ranged = false, air = false) {
  const view = seatView();
  if (!view) return;
  st().setMode('normal');
  st().set({ warPrompt: { seat: st().seat, view, unit, target, team, ranged, air } });
}

/**
 * Confirmed in the Declare War dialog: queue the declaration and the attack together. A ranged
 * attack can't wait for a War that only starts next turn, so then only the declaration is queued.
 */
export function declareWarAndAttack(unit: number | null, target: Hex | null, team: TeamId, ranged = false, air = false) {
  const view = seatView();
  if (!view) return;
  const war = warDeclaration(view, team);
  st().set({ warPrompt: null });
  if (!('action' in war)) return;
  const now = war.action.effective_turn === view.turn;
  const attack: Order | null = unit === null || target === null || (ranged && !now) ? null
    : air ? { type: 'AirStrike', plane: unit, target }
    // A plain MoveUnit: the core pathfinds once War opens the borders and follows the unit if it moves.
    : ranged ? { type: 'RangedAttack', unit, target } : { type: 'MoveUnit', unit, to: target };
  const name = empireTitle(view, team);
  // G17: the decision is confirmed over the map, with Undo.
  answerDecision(attack ? `War declared on ${name}; the attack goes in ${now ? 'this' : 'next'} turn`
    : `War declared on ${name}; it starts ${now ? 'this' : 'next'} turn`,
  (orders) => {
    const declared = queueDiplomacy(orders, war.action.order, view);
    return !attack ? declared : attack.type === 'AirStrike' ? setAirOrder(declared, attack) : setUnitOrder(declared, attack);
  }, undefined, attack ? 'commitAttack' : 'commit');
  if (!attack) return;
  if (air) return;
  const selected = st().selection;
  const plan = st().orders;
  setTimeout(() => { if (st().selection === selected && st().orders === plan) st().nextUnit(); }, 120);
}

/** Cancelled: keep the Peace. An attack already queued on that target (now invalid) is dropped. */
export function cancelWarPrompt() {
  const prompt = st().warPrompt;
  st().set({ warPrompt: null });
  if (!prompt || prompt.unit === null || !prompt.target) return;
  const { unit, target } = prompt;
  if (prompt.air) return;
  st().editOrders((orders) => {
    const o = pendingUnitOrder(orders, unit);
    const aimed = o && (o.type === 'RangedAttack' ? hexEq(o.target, target) : (o.type === 'MoveUnit' || o.type === 'MoveUnitPath') && hexEq(o.to, target));
    return aimed ? clearUnitOrder(orders, unit) : orders;
  });
}

/** What core would say about this one unit order before it is queued (local clients; online validates on the host). */
function orderProblem(order: Order): string | null {
  const s = st();
  const view = seatView();
  if (!view || !s.client || s.client.kind === 'remote') return null;
  const next = setUnitOrder(s.orders[s.seat] ?? [], order);
  try {
    return s.client.validate(view.me, next).find((e) => e.index === next.length - 1)?.message ?? null;
  } catch {
    return null;
  }
}

function issue(order: Order, advance = true) {
  const view = seatView();
  const attack = order.type === 'RangedAttack' || ((order.type === 'MoveUnit' || order.type === 'MoveUnitPath') && view && enemyAt(view, order.to));
  st().editOrders((orders) => setUnitOrder(orders, order), attack ? 'commitAttack' : 'commit');
  if (st().mode === 'move') st().setMode('normal');
  // Advance only if the player hasn't selected something else meanwhile (e.g. F then a quick Return).
  const selected = st().selection;
  const plan = st().orders;
  if (advance) setTimeout(() => { if (st().selection === selected && st().orders === plan) st().nextUnit(); }, 120);
}

/** Occupation lock: a locked unit cannot leave its city, so a move or disband is refused at the click with the reason and nothing is queued. */
function lockedRefusal(u: UnitView): boolean {
  const view = seatView();
  const why = view && occupationRefusal(view, u);
  if (!why) return false;
  st().set({ error: `${why}.` });
  feel.invalid();
  return true;
}

export function moveSelected(to: Hex) {
  const u = selectedOwnUnit();
  if (!u) return;
  if (hexEq(u.pos, to)) {
    st().editOrders((o) => clearUnitOrder(o, u.id));
    return;
  }
  if (st().mode === 'ranged') {
    rangedSelected(to);
    return;
  }
  if (lockedRefusal(u)) return;
  const view = seatView()!;
  const peace = u.military ? peacefulAt(view, to) : null;
  if (peace) {
    // Never a silent bounce: attacking a team at Peace is a decision, asked for explicitly.
    if (warQueuedNow(view, st().orders[st().seat] ?? [], peace.team)) issue({ type: 'MoveUnit', unit: u.id, to });
    else promptWar(peace.team, u.id, to);
    return;
  }
  // Reissuing an order recalculates its route. The map shows a confirmed path as submitted,
  // but feeding that old path back into previewPath would make a detour impossible to correct.
  const otherOrders = (st().orders[st().seat] ?? []).filter((o) => !('unit' in o && o.unit === u.id));
  const path = previewPath(view, u.pos, to, otherOrders, u.id, { avoidEnemyReach: true });
  issue(path && path.length > 1 ? { type: 'MoveUnitPath', unit: u.id, to, path } : { type: 'MoveUnit', unit: u.id, to });
}

export function rangedSelected(target: Hex) {
  const u = selectedOwnUnit();
  if (!u || u.ranged <= 0) return;
  if (distance(u.pos, target) > u.range) {
    // A mounted archer (the Keshig) can ride at the target instead: say so, never leave the player at a dead end.
    st().set({ error: `Target is out of range: this unit reaches ${u.range} ${u.range === 1 ? 'tile' : 'tiles'}.${attacksInMelee(u.kind) ? ' Right-click it to charge instead.' : ''}` });
    feel.invalid();
    return;
  }
  const view = seatView()!;
  const blocked = shotBlockReason(view, u.pos, target, u.kind);
  if (blocked) {
    st().set({ error: blocked });
    feel.invalid();
    return;
  }
  const enemy = rangedEnemyAt(view, target);
  const peace = enemy ? null : peacefulRangedAt(view, target);
  if (peace && !warQueuedNow(view, st().orders[st().seat] ?? [], peace.team)) {
    // Never an order that looks accepted and then fails: attacking a team at Peace asks first.
    promptWar(peace.team, u.id, target, true);
    return;
  }
  const order: Order = { type: 'RangedAttack', unit: u.id, target };
  const problem = !enemy && !peace ? 'Nothing to fire on there: pick an enemy military unit or city in range.' : orderProblem(order);
  if (problem) {
    // Nothing is queued, and targeting stays on so another tile can be picked.
    st().set({ error: `${problem.charAt(0).toUpperCase()}${problem.slice(1)}${/[.!?]$/.test(problem) ? '' : '.'}` });
    feel.invalid();
    return;
  }
  st().setMode('normal');
  issue(order);
}

export function sleepSelected() {
  const u = selectedOwnUnit();
  if (u) issue({ type: 'SleepUnit', unit: u.id });
}

/**
 * Why a unit cannot take the Heal order now, in plain words, or null when it can. Mirrors core's
 * `units::check_heal` so the button and the H key say why before an order is queued.
 */
export function healBlocker(u: UnitView): string | null {
  if (u.hp >= 100) return 'Already at full health: nothing to heal.';
  if (u.resting_heal && u.resting_heal.total <= 0) return 'Closed foreign territory: nothing heals here. Leave it first.';
  return null;
}

/** Turns a resting unit needs to reach full HP at its current rate (null: it does not heal here). */
export function turnsToHeal(u: UnitView): number | null {
  const rate = u.resting_heal?.total ?? 0;
  return u.hp >= 100 || rate <= 0 ? null : Math.ceil((100 - u.hp) / rate);
}

/** What Heal would do where the unit stands: the rate with its sources, and the turns to full (null when it heals nothing). */
export function healForecast(u: UnitView): string | null {
  const turns = turnsToHeal(u);
  if (!turns || !u.resting_heal) return null;
  const parts = u.resting_heal.parts.filter(([, n]) => n !== 0);
  // A heal that is all home territory reads as where it happens; any other mix keeps its sources.
  const home = parts.length === 1 && parts[0][0] === 'Home territory';
  const sources = parts.map(([label, n]) => `${label} ${n > 0 ? '+' : ''}${n}`).join(', ');
  return `Heals ${u.resting_heal.total} HP a turn ${home ? 'in your territory' : `here${sources ? ` (${sources})` : ''}`}: full in ${turns} ${turns === 1 ? 'turn' : 'turns'}.`;
}

/** Rest like Fortify or Wait, but wake and ask for orders the turn HP is full (H). */
export function healSelected() {
  const u = selectedOwnUnit();
  if (!u) return;
  const blocked = healBlocker(u);
  if (blocked) {
    st().set({ error: blocked });
    feel.invalid();
    return;
  }
  issue({ type: 'SleepUnit', unit: u.id, until_healed: true });
}

export function toggleAutoFireSelected() {
  const u = selectedOwnUnit();
  const v = seatView();
  // Only a ranged unit standing in one of our cities can auto-fire (the V key reaches here for any unit).
  if (u && v && u.ranged > 0 && v.cities.some((c) => c.owner === v.me && hexEq(c.pos, u.pos))) {
    const pending = st().orders[st().seat].find((o) => o.type === 'SetAutoFire' && o.unit === u.id);
    const enabled = pending?.type === 'SetAutoFire' ? !pending.enabled : u.standing_order !== 'AutoFire';
    issue({ type: 'SetAutoFire', unit: u.id, enabled });
  }
}

export function foundCitySelected() {
  const u = selectedOwnUnit();
  const v = seatView();
  // The button shows why it is disabled; the B key and map menu follow the same rule.
  if (u && v && !foundCityUnitBlocker(v, u)) {
    // A city is about to occupy this tile, so the advance skips whatever stands on it: selecting a
    // stacked escort would turn the next click on the new city into a move. Until then nothing is selected.
    const tile = u.pos, settler = u.id;
    issue({ type: 'FoundCity', unit: settler }, false);
    st().set({ selection: null });
    const plan = st().orders;
    setTimeout(() => {
      const now = seatView();
      if (!now || st().selection !== null || st().orders !== plan) return;
      const rest = unitsNeedingOrders(now, st().orders[st().seat] ?? []).filter((x) => !hexEq(x.pos, tile));
      const next = rest.find((x) => x.id > settler) ?? rest[0];
      if (next) st().select({ kind: 'unit', id: next.id }, true);
    }, 120);
  }
}

export function disbandSelected() {
  const u = selectedOwnUnit();
  if (!u || lockedRefusal(u)) return;
  // The Heir carries Byzantium's 2 VP: disbanding him forfeits it, so the panel asks first.
  if (u.kind === 'Heir' && pendingUnitOrder(st().orders[st().seat], u.id)?.type !== 'DisbandUnit') { st().set({ heirDisbandAsk: u.id }); return; }
  issue({ type: 'DisbandUnit', unit: u.id });
}

/** The Heir's Disband, once confirmed in the panel. */
export function confirmDisbandHeir() {
  const u = selectedOwnUnit();
  st().set({ heirDisbandAsk: null });
  if (u) issue({ type: 'DisbandUnit', unit: u.id });
}

export function upgradeSelected() {
  const u = selectedOwnUnit();
  if (u && u.upgrade_cost !== null) issue({ type: 'UpgradeUnit', unit: u.id }, false);
}

/** Buy Siege for the selected unit (core `Order::UpgradeSiege`). */
export function siegeSelected() {
  const u = selectedOwnUnit();
  if (u && u.siege_offer && !u.siege_offer.blocker) issue({ type: 'UpgradeSiege', unit: u.id }, false);
}

export function cancelSelected() {
  const u = selectedOwnUnit();
  if (u) st().editOrders((o) => clearUnitOrder(o, u.id));
}

export function skipSelected() {
  const u = selectedOwnUnit();
  if (!u) return;
  issue({ type: 'SkipUnit', unit: u.id }, false);
  st().nextUnit();
}

export function toggleRangedMode() {
  const u = selectedOwnUnit();
  if (!u || u.ranged <= 0) return;
  st().setMode(st().mode === 'ranged' ? 'normal' : 'ranged');
}

/** User intent only: automatic observer acknowledgements must remain quiet. */
export function readySelected(seat = st().seat) {
  const before = st().ready[seat];
  const result = st().toggleReady(seat);
  const feedback = () => {
    const after = st().ready[seat];
    if (after !== before) feel.ready(after);
  };
  // Normal readiness stays synchronous. Planning may first need to initialize WASM.
  if (st().busy === 'Planning automation…') return result.then(feedback);
  feedback();
  return result;
}

export function toggleAutoExploreSelected() {
  const u = selectedOwnUnit();
  if (!u || !canAutoExplore(u.kind) || st().busy) return;
  const exploring = isExploring(u, st().orders[st().seat]);
  if (!exploring && lockedRefusal(u)) return;
  const automation = exploring ? null : 'Explore';
  st().editOrders((orders) => setUnitOrder(orders, { type: 'SetAutomation', unit: u.id, automation }));
  feel.select();
}

/** Explicit mouse targeting uses the same preview and validated move/attack command as right-click. */
export function toggleMoveSelected() {
  const u = selectedOwnUnit();
  if (!u || (st().mode !== 'move' && lockedRefusal(u))) return;
  st().setMode(st().mode === 'move' ? 'normal' : 'move');
  document.querySelector<HTMLElement>('[data-game-map]')?.focus({ preventScroll: true });
}
