/** The advisor explains one decision from a Hard plan made from this seat's own view. */
import { create } from 'zustand';
import { advisorOrders } from '../automation';
import { buildingStatsFor, itemKey, itemName, TECH_NAME } from '../data';
import { isThreat } from '../diplomacy';
import { distance, hexEq } from '../hex';
import { effectiveResearchQueue, pendingResearchQueue } from '../orders';
import { pendingDecisions } from '../turnFlow';
import { HOLD_TURNS } from '../victoryHold';
import { techCardUnlocks } from './techCardUnlocks';
import { YIELD_LABEL, type YieldKey } from './yieldKeys';
import type { BattleForecast, CityView, Hex, Order, OrderError, PlayerView, UnitView } from '../types';

/** Keep the old key: disabling first-game hints also disables the advisor. */
export const ADVISOR_KEY = 'legatum.first-game-hints.v2';
export type AdvisorTarget = { kind: 'tile'; hex: Hex } | { kind: 'control'; name: 'research' | 'turn' | 'race' | 'found' | 'fortify' | 'explore' | 'skip' | 'ranged' | 'build' | 'tech' | 'closeResearch'; value?: string } | null;
export interface AdvisorStep { instruction: string; key?: string; target: AdvisorTarget }
export interface Suggestion { id: string; title: string; why: string; steps: AdvisorStep[]; current: number; acknowledge?: boolean }
export interface AdvisorContext {
  view: PlayerView;
  orders: Order[];
  errors?: OrderError[];
  selection: { kind: string; id: number } | null;
  humans: number;
  /** An online game: the turn button reads Ready even with one human in this browser. */
  remote?: boolean;
  touch: boolean;
  techOpen?: boolean;
  mode?: string;
  local?: Parameters<typeof pendingDecisions>[3];
  forecast?: (unit: number, at: Hex, mode: 'Melee' | 'Ranged') => BattleForecast | null;
  /** Supplying a plan makes the decision logic pure and easy to test. Undefined reads the loaded WASM. */
  plan?: Order[] | null;
  turnLabel?: string;
}
const RESEARCH_FALLBACK = 'Research gives your empire new choices.';
const ownCities = (v: PlayerView) => v.cities.filter((c) => c.owner === v.me);
const ownUnits = (v: PlayerView) => v.units.filter((u) => u.owner === v.me);
const factionOf = (v: PlayerView) => v.players.find((p) => p.id === v.me)?.faction ?? 'Switzerland';
const otherSide = (v: PlayerView, owner: number) => owner !== v.me && v.players.find((p) => p.id === owner)?.team !== v.team;
const nearestCity = (v: PlayerView, at: Hex) => ownCities(v).sort((a, b) => distance(a.pos, at) - distance(b.pos, at))[0];
const validOrders = (ctx: AdvisorContext) => ctx.orders.filter((_, i) => !ctx.errors?.some((e) => e.index === i));

/** One honest reason drawn from visible facts. Forecast odds appear only when the client supplies them. */
export function advisorWhy(order: Order, view: PlayerView, city?: CityView, unit?: UnitView, forecast?: AdvisorContext['forecast']): string {
  if ((order.type === 'SetProduction' || order.type === 'SetProductionQueue') && city) {
    const item = order.type === 'SetProduction' ? order.item : order.queue[0];
    if (!item) return `${city.name} needs a build to use its Production.`;
    if (item.kind === 'Unit' && item.unit === 'Settler') return 'A Settler founds a new city, and more cities grow your empire.';
    if (item.kind === 'Unit' && item.unit === 'Scout') return 'A Scout can find city sites, ruins and rivals.';
    if (item.kind === 'Unit') {
      const near = view.units.filter((u) => u.military && isThreat(view, u.owner) && distance(u.pos, city.pos) <= 6);
      if (near.length) return `Enemy units are near ${city.name}. A military unit can defend it.`;
      const units = ownUnits(view).filter((u) => u.military).length, cities = ownCities(view).length;
      if (!units) return `You have no military units guarding ${city.name}.`;
      return `You have ${units} military ${units === 1 ? 'unit' : 'units'} for ${cities} ${cities === 1 ? 'city' : 'cities'}.`;
    }
    if (item.kind === 'Building') {
      const yields = buildingStatsFor(factionOf(view), item.building).yields;
      const gain = (['food', 'production', 'gold', 'science'] as YieldKey[]).find((key) => (yields[key] ?? 0) > 0);
      if (gain) return `${itemName(item, factionOf(view))} adds ${yields[gain]} ${YIELD_LABEL[gain]} in ${city.name}.`;
      return `${itemName(item, factionOf(view))} helps ${city.name}.`;
    }
    return `${city.name} can put its Production to work.`;
  }
  if (order.type === 'SetResearch' || order.type === 'SetResearchQueue') {
    const tech = order.type === 'SetResearch' ? order.tech : order.techs[0];
    const unlock = tech && techCardUnlocks(view.research, tech, factionOf(view)).find((u) => u.kind !== 'Effect');
    if (!unlock || !tech) return RESEARCH_FALLBACK;
    const main = unlock.yields.find(([key]) => key === 'food' || key === 'production' || key === 'gold' || key === 'science');
    return `${TECH_NAME[tech]} unlocks the ${unlock.label}${main ? `: +${main[1]} ${YIELD_LABEL[main[0]]} in a city` : ''}.`;
  }
  if (order.type === 'SetAutomation') return 'Exploring finds city sites, ruins and rivals.';
  if (order.type === 'FoundCity') return 'A new city gives your empire more room to grow.';
  if (order.type === 'SleepUnit') return unit?.military && unit.kind !== 'Scout' ? 'Fortified units defend better and heal.' : 'Waiting keeps this unit in place until you give another order.';
  if (order.type === 'SkipUnit') return 'This unit can wait until the next turn.';
  if ((order.type === 'MoveUnit' || order.type === 'MoveUnitPath' || order.type === 'RangedAttack') && unit) {
    const at = order.type === 'RangedAttack' ? order.target : order.to;
    const enemy = view.units.find((u) => hexEq(u.pos, at) && otherSide(view, u.owner));
    const enemyCity = view.cities.find((c) => hexEq(c.pos, at) && otherSide(view, c.owner));
    if (enemy || enemyCity) {
      let odds: number | undefined;
      try { odds = forecast?.(unit.id, at, order.type === 'RangedAttack' ? 'Ranged' : 'Melee')?.win_chance; } catch { /* No forecast is safer than guessed odds. */ }
      if (odds !== undefined && odds !== null) return `You have about a ${odds}% chance to win this battle.`;
      return `This attacks ${enemyCity?.name ?? 'an enemy unit'}.`;
    }
    if (unit.kind === 'Settler') return 'This is a good place for a city.';
    if (unit.kind === 'Scout') return 'Exploring finds city sites, ruins and rivals.';
    const home = nearestCity(view, at);
    if (home && distance(home.pos, at) <= 2 && view.units.some((u) => u.military && isThreat(view, u.owner) && distance(u.pos, home.pos) <= 6)) return `Guard ${home.name}: enemy units are near.`;
    if (home) return `Move closer to ${home.name}.`;
    return 'Move to explore or protect your empire.';
  }
  return 'This is your best move right now.';
}

const tile = (hex: Hex): AdvisorTarget => ({ kind: 'tile', hex });
const control = (name: Exclude<NonNullable<AdvisorTarget>, { kind: 'tile' }>['name'], value?: string): AdvisorTarget => ({ kind: 'control', name, value });
const unitName = (v: PlayerView, u: UnitView) => itemName({ kind: 'Unit', unit: u.kind }, factionOf(v));
const selected = (ctx: AdvisorContext, kind: string, id: number) => ctx.selection?.kind === kind && ctx.selection.id === id;

function citySuggestion(ctx: AdvisorContext, city: CityView, order?: Extract<Order, { type: 'SetProduction' | 'SetProductionQueue' }>): Suggestion {
  const item = order?.type === 'SetProduction' ? order.item : order?.queue[0];
  const name = item && itemName(item, factionOf(ctx.view));
  return { id: `city:${city.id}`, title: name ? `Build ${name} in ${city.name}` : `Choose a build in ${city.name}`,
    why: order ? advisorWhy(order, ctx.view, city) : `${city.name} needs a build to use its Production.`,
    current: selected(ctx, 'city', city.id) ? 1 : 0,
    steps: [
      { instruction: `${ctx.touch ? 'Tap' : 'Click'} ${city.name}.`, target: tile(city.pos) },
      { instruction: name ? `${ctx.touch ? 'Tap' : 'Click'} Build on ${name}.` : 'Choose a unit or building to build.', target: item ? control('build', itemKey(item)) : null },
    ] };
}
function researchSuggestion(ctx: AdvisorContext, order?: Extract<Order, { type: 'SetResearch' | 'SetResearchQueue' }>): Suggestion {
  const planned = order?.type === 'SetResearch' ? order.tech : order?.techs[0];
  const tech = ctx.view.research.available.some((option) => option.tech === planned) ? planned : undefined;
  const name = tech && TECH_NAME[tech];
  return { id: 'research', title: name ? `Research ${name}` : 'Choose research',
    why: order && tech ? advisorWhy(order, ctx.view) : RESEARCH_FALLBACK, current: ctx.techOpen ? 1 : 0,
    steps: [
      { instruction: `${ctx.touch ? 'Tap' : 'Click'} Research at the top.`, key: ctx.touch ? undefined : 'T', target: control('research') },
      { instruction: name ? `${ctx.touch ? 'Tap' : 'Click'} Research now on ${name}.` : 'Choose a technology.', target: tech ? control('tech', tech) : null },
    ] };
}
function unitSuggestion(ctx: AdvisorContext, unit: UnitView, order?: Order): Suggestion | null {
  const name = unitName(ctx.view, unit), first: AdvisorStep = { instruction: `${ctx.touch ? 'Tap' : 'Click'} your ${name}.`, target: tile(unit.pos) };
  const current = selected(ctx, 'unit', unit.id) ? 1 : 0;
  const base = { id: `unit:${unit.id}`, current, steps: [first], why: order ? advisorWhy(order, ctx.view, undefined, unit, ctx.forecast) : 'Each unit needs an order this turn.' };
  if (!order) return { ...base, title: `Give your ${name} an order`, steps: [first, { instruction: `${ctx.touch ? 'Tap' : 'Click'} Fortify, or move it, or press Space to skip.`, target: control('fortify') }] };
  if (order.type === 'MoveUnit' || order.type === 'MoveUnitPath' || order.type === 'RangedAttack') {
    const at = order.type === 'RangedAttack' ? order.target : order.to;
    const enemy = ctx.view.units.find((u) => hexEq(u.pos, at) && otherSide(ctx.view, u.owner));
    const targetCity = ctx.view.cities.find((c) => hexEq(c.pos, at) && otherSide(ctx.view, c.owner));
    const targetName = targetCity?.name ?? (enemy ? unitName(ctx.view, enemy) : 'marked tile');
    const ranged = order.type === 'RangedAttack';
    if (ranged) return { ...base, title: `Attack with your ${name}`, current: !selected(ctx, 'unit', unit.id) ? 0 : ctx.mode === 'ranged' ? 2 : 1,
      steps: [first, { instruction: `${ctx.touch ? 'Tap' : 'Click'} Ranged Attack.`, key: ctx.touch ? undefined : 'R', target: control('ranged') },
        { instruction: `${ctx.touch ? 'Tap' : 'Click'} the ${targetName}.`, target: tile(at) }] };
    return { ...base, title: `${ranged || enemy || targetCity ? 'Attack' : 'Move'} with your ${name}`,
      steps: [first, { instruction: ctx.touch ? `Tap the ${targetName}, then tap it again.` : `Right-click the ${targetName}. On a trackpad, click with two fingers.`, target: tile(at) }] };
  }
  const actions: Partial<Record<Order['type'], { title: string; instruction: string; key?: string; target: AdvisorTarget }>> = {
    SleepUnit: { title: `Fortify your ${name}`, instruction: unit.military && unit.kind !== 'Scout' ? `${ctx.touch ? 'Tap' : 'Click'} Fortify.` : `${ctx.touch ? 'Tap' : 'Click'} Wait.`, key: 'F', target: control('fortify') },
    SkipUnit: { title: `Skip your ${name}`, instruction: 'Press Space to skip this turn.', key: 'Space', target: control('skip') },
    FoundCity: { title: 'Found a new city', instruction: `${ctx.touch ? 'Tap' : 'Click'} Found City.`, key: 'B', target: control('found') },
    SetAutomation: order.type === 'SetAutomation' && order.automation === 'Explore' ? { title: `Explore with your ${name}`, instruction: `${ctx.touch ? 'Tap' : 'Click'} Auto-explore.`, key: 'E', target: control('explore') } : undefined,
  };
  const action = actions[order.type];
  return action ? { ...base, title: action.title, steps: [first, { instruction: action.instruction, key: ctx.touch ? undefined : action.key, target: action.target }] } : null;
}

export function advisorSuggestion(ctx: AdvisorContext, seen: readonly string[], skipped: readonly string[]): Suggestion | null {
  const v = ctx.view;
  if (v.finished || v.players.find((p) => p.id === v.me)?.alive === false) return null;
  if (v.turn === 1 && !seen.includes('welcome')) return { id: 'welcome', title: 'Your advisor will guide you', why: "I'll suggest what to do each turn and show you where to click.", steps: [], current: 0, acknowledge: true };
  if (v.turn > 1 && seen.includes('welcome') && !seen.includes('report')) return { id: 'report', title: 'Read the Turn report', why: 'The Turn report lists battles, discoveries and failed orders.', steps: [{ instruction: 'Press L to open it.', key: ctx.touch ? undefined : 'L', target: null }], current: 0, acknowledge: true };
  if (v.turn >= 8 && !seen.includes('race')) return { id: 'race', title: 'Win the victory race', why: `Reach your team’s VP target and hold it for ${HOLD_TURNS} turns to win.`, steps: [{ instruction: 'Look at the Victory race.', target: control('race') }], current: 0, acknowledge: true };
  const plan = ctx.plan === undefined ? advisorOrders(v) : ctx.plan;
  const orders = validOrders(ctx);
  const decisions = pendingDecisions(v, orders, ctx.errors ?? [], ctx.local);
  // Research is chosen but its panel is still open: closing it is the next step, since the rest of the turn is behind it.
  // Only after a choice made this turn, so opening Research later to browse isn't nagged; "Not now" dismisses it.
  const chosen = pendingResearchQueue(orders);
  if (ctx.techOpen && chosen?.length && !skipped.includes('closeResearch')) {
    const tech = TECH_NAME[chosen[0]];
    return { id: 'closeResearch', title: 'Close Research', why: tech ? `${tech} is chosen. Your other decisions are on the map.` : 'Your research is chosen. Your other decisions are on the map.',
      steps: [{ instruction: `${ctx.touch ? 'Tap' : 'Click'} Close.`, key: ctx.touch ? undefined : 'Esc', target: control('closeResearch') }], current: 0 };
  }
  const handled = (kind: string) => kind === 'city' || kind === 'unit' || kind === 'research';
  const firstHandled = decisions.findIndex((d) => handled(d.kind));
  const firstUnanswered = decisions.findIndex((d, i) => !handled(d.kind) && (firstHandled < 0 || i < firstHandled) && !skipped.includes(`answer:${d.kind}:${'id' in d ? d.id : ''}`));
  if (firstUnanswered >= 0) {
    const d = decisions[firstUnanswered], id = `answer:${d.kind}:${'id' in d ? d.id : ''}`;
    return { id, title: 'Answer this first', why: d.label, current: 0,
      steps: [{ instruction: `${ctx.touch ? 'Tap' : 'Click'} ${ctx.turnLabel ?? d.label}.`, target: control('turn') }] };
  }
  // Follow the screen: the open Research panel or the selected city or unit is the decision in hand.
  const inHand = (d: (typeof decisions)[number]) => (d.kind === 'research' && ctx.techOpen)
    || ((d.kind === 'city' || d.kind === 'unit') && ctx.selection?.kind === d.kind && ctx.selection.id === d.id);
  for (const d of [...decisions.filter(inHand), ...decisions.filter((d) => !inHand(d))]) {
    if (d.kind === 'city') {
      const city = ownCities(v).find((c) => c.id === d.id);
      if (!city || skipped.includes(`city:${d.id}`)) continue;
      const choice = plan?.find((o): o is Extract<Order, { type: 'SetProduction' | 'SetProductionQueue' }> => (o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city.id);
      return citySuggestion(ctx, city, choice);
    }
    if (d.kind === 'research') {
      if (skipped.includes('research') || effectiveResearchQueue(orders, v.research).length) continue;
      const choice = plan?.find((o): o is Extract<Order, { type: 'SetResearch' | 'SetResearchQueue' }> => o.type === 'SetResearch' || o.type === 'SetResearchQueue');
      return researchSuggestion(ctx, choice);
    }
    if (d.kind === 'unit') {
      if (skipped.includes(`unit:${d.id}`)) continue;
      const unit = ownUnits(v).find((u) => u.id === d.id);
      if (!unit) continue;
      const choice = plan?.find((o) => 'unit' in o && o.unit === unit.id);
      const suggestion = unitSuggestion(ctx, unit, choice) ?? unitSuggestion(ctx, unit);
      if (suggestion) return suggestion;
    }
  }
  if (decisions.length) return null;
  const solo = ctx.humans <= 1 && !ctx.remote; // Matches the turn button's label (SeatBar).
  return { id: 'endTurn', title: solo ? 'End the turn' : 'Ready for the turn', why: 'Your orders are set. Every empire’s orders happen together.',
    steps: [{ instruction: `${ctx.touch ? 'Tap' : 'Click'} ${solo ? 'End turn' : 'Ready'}.`, key: ctx.touch ? undefined : 'Return', target: control('turn', 'end') }], current: 0 };
}

interface AdvisorPreferences { enabled: boolean; seen: string[] }
export function readAdvisorPreferences(): AdvisorPreferences {
  try {
    const raw: unknown = JSON.parse(globalThis.localStorage?.getItem(ADVISOR_KEY) ?? 'null');
    if (raw && typeof raw === 'object') {
      const p = raw as { enabled?: unknown; seen?: unknown; tutorial?: unknown };
      return { enabled: p.enabled !== false, seen: Array.isArray(p.seen) ? p.seen.filter((x): x is string => typeof x === 'string') : Array.isArray(p.tutorial) ? p.tutorial.filter((x): x is string => typeof x === 'string') : [] };
    }
  } catch { /* Storage may be unavailable. */ }
  return { enabled: true, seen: [] };
}
function savePreferences(p: AdvisorPreferences) {
  try { globalThis.localStorage?.setItem(ADVISOR_KEY, JSON.stringify(p)); } catch { /* Session state still works. */ }
}
interface AdvisorState extends AdvisorPreferences {
  skippedTurn: number;
  skipped: string[];
  markSeen(id: string): void;
  skip(id: string, turn: number): void;
  setEnabled(enabled: boolean): void;
}
export const useAdvisor = create<AdvisorState>((set) => ({
  ...readAdvisorPreferences(), skippedTurn: -1, skipped: [],
  markSeen: (id) => set((s) => { if (s.seen.includes(id)) return s; const next = { ...s, seen: [...s.seen, id] }; savePreferences(next); return next; }),
  skip: (id, turn) => set((s) => ({ skippedTurn: turn, skipped: [...(s.skippedTurn === turn ? s.skipped : []), id] })),
  setEnabled: (enabled) => set((s) => { const next = { ...s, enabled }; savePreferences(next); return next; }),
}));
