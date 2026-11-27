/** The advisor explains one decision from a Hard plan made from this seat's own view. */
import { create } from 'zustand';
import { advisorOrders } from '../automation';
import { buildingStatsFor, effectiveTurnLimit, itemKey, itemName, TECH_NAME, wallsRepairing, wallsUpgradeName } from '../data';
import { WALLS_HP_PER_LEVEL } from '../combat';
import { isThreat } from '../diplomacy';
import { ownerName } from '../barbarians';
import { distance, hexEq } from '../hex';
import { effectiveResearchQueue, pendingResearchQueue } from '../orders';
import { revealedByTech } from '../resources';
import { pendingDecisions } from '../turnFlow';
import { HOLD_TURNS } from '../victoryHold';
import { techCardUnlocks } from './techCardUnlocks';
import { YIELD_LABEL, type YieldKey } from './yieldKeys';
import { strategyTip, tipKind, type TipLog } from './advisorTips';
import type { LockLog } from './advisorTips';
import { occupiedCityName } from './forecastWarnings';
import type { BattleForecast, CityView, Hex, Order, OrderError, PlayerView, TechId, UnitView } from '../types';

/** Keep the old key: disabling first-game hints also disables the advisor. */
export const ADVISOR_KEY = 'legatum.first-game-hints.v2';
export type AdvisorTarget = { kind: 'tile'; hex: Hex } | { kind: 'control'; name: 'research' | 'turn' | 'race' | 'diplomacy' | 'found' | 'fortify' | 'explore' | 'skip' | 'ranged' | 'build' | 'tech' | 'queueTech' | 'researchQueue' | 'closeResearch' | 'buyTile' | 'buyCity'; value?: string } | null;
export interface AdvisorStep { instruction: string; key?: string; target: AdvisorTarget }
export interface Suggestion { id: string; title: string; why: string; steps: AdvisorStep[]; current: number; acknowledge?: boolean; tip?: boolean }
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
  /** The city whose Buy a tile placement is open, if any: the advisor guides the last click there. */
  placingTile?: number | null;
  /** The latest resolved turn finished a technology and left no next technology. */
  researchQueueDrained?: boolean;
}
const RESEARCH_FALLBACK = 'Research gives your empire new choices.';
const ownCities = (v: PlayerView) => v.cities.filter((c) => c.owner === v.me);
const ownUnits = (v: PlayerView) => v.units.filter((u) => u.owner === v.me);
const factionOf = (v: PlayerView) => v.players.find((p) => p.id === v.me)?.faction ?? 'Switzerland';
const otherSide = (v: PlayerView, owner: number) => owner !== v.me && v.players.find((p) => p.id === owner)?.team !== v.team;
// Match the technologies the Hard planner prioritizes under threat (`crates/ai/src/research.rs::is_military_tech`).
const DEFENCE_TECHS = new Set<TechId>(['Masonry', 'Mathematics', 'Feudalism', 'Machinery', 'Chivalry', 'Gunpowder', 'Metallurgy', 'MilitaryEngineering', 'Rifling', 'MilitaryScience', 'Cannonry', 'ReplaceableParts', 'Combustion', 'Steel', 'Flight', 'Robotics', 'StealthTechnology']);
function researchWhy(order: Extract<Order, { type: 'SetResearch' | 'SetResearchQueue' }>, view: PlayerView): string {
  const tech = order.type === 'SetResearch' ? order.tech : order.techs[0];
  if (!tech) return RESEARCH_FALLBACK;
  const unlocks = techCardUnlocks(view.research, tech, factionOf(view));
  const military = DEFENCE_TECHS.has(tech) && unlocks.find((u) => u.kind === 'Unit' || u.label === 'Walls');
  const nearArmy = view.units.some((u) => u.military && otherSide(view, u.owner) && ownCities(view).some((c) => distance(c.pos, u.pos) <= 6)
    && (isThreat(view, u.owner) || (view.players.find((p) => p.id === u.owner)?.army ?? 0) > (view.players.find((p) => p.id === view.me)?.army ?? 0)));
  const ahead = view.players.some((p) => p.alive && otherSide(view, p.id) && p.era > view.research.era + 1
    && view.cities.some((c) => c.owner === p.id && ownCities(view).some((own) => distance(c.pos, own.pos) <= 12)));
  if (military && (nearArmy || ahead)) return `${nearArmy ? 'A foreign military unit is near one of your cities' : 'A rival is ahead in research'}: ${TECH_NAME[tech]} unlocks ${military.label}.`;
  const resource = revealedByTech(tech)[0];
  if (resource) return `${TECH_NAME[tech]} shows ${resource} on the map.`;
  const unlock = unlocks.find((u) => u.kind !== 'Effect');
  if (!unlock) return RESEARCH_FALLBACK;
  const main = unlock.yields.find(([key]) => key === 'food' || key === 'production' || key === 'gold' || key === 'science');
  return `${TECH_NAME[tech]} unlocks the ${unlock.label}${main ? `: +${main[1]} ${YIELD_LABEL[main[0]]} in a city` : ''}.`;
}
const nearestCity = (v: PlayerView, at: Hex) => ownCities(v).sort((a, b) => distance(a.pos, at) - distance(b.pos, at))[0];
/**
 * Routine steps (moving a unit, Fortify, Skip, Auto-explore, End turn, Close Research) are taught only until the
 * player has done each on PRACTICE_TURNS different turns; after that the advisor stays quiet about them (user, 2026-11-25:
 * "it doesn't need to show me where to move my unit every time"). Builds, research, Settlers, Found City and attacks
 * are advice rather than controls, so they always show.
 */
export const PRACTICE_TURNS = 3;
export type PracticeKind = 'move' | 'fortify' | 'skip' | 'explore' | 'endTurn' | 'closeResearch';
export type Practice = Readonly<Partial<Record<PracticeKind, number>>>;
/** The routine a unit order practises, or null when the order is advice worth showing every time. */
export function practiceKind(order: Order | undefined, view: PlayerView): PracticeKind | null {
  if (!order) return 'move';
  switch (order.type) {
    case 'MoveUnit': case 'MoveUnitPath': {
      const unit = view.units.find((u) => u.id === order.unit);
      if (!unit || unit.kind === 'Settler') return null;
      const attack = view.units.some((u) => hexEq(u.pos, order.to) && otherSide(view, u.owner)) || view.cities.some((c) => hexEq(c.pos, order.to) && otherSide(view, c.owner));
      return attack ? null : 'move';
    }
    case 'SleepUnit': return 'fortify';
    case 'SkipUnit': return 'skip';
    case 'SetAutomation': return order.automation === 'Explore' ? 'explore' : null;
    default: return null;
  }
}
const learned = (practice: Practice, kind: PracticeKind | null) => kind !== null && (practice[kind] ?? 0) >= PRACTICE_TURNS;
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
    // No Walls work finishes while the city is under attack (core `walls_work_blocked`).
    if (item.kind === 'Building' && (item.building === 'UpgradeWalls' || item.building === 'Walls') && city.detail?.walls_blocked) return `Walls can't be finished while ${city.name} is under attack. Production waits.`;
    if (item.kind === 'Building' && item.building === 'UpgradeWalls') return (city.walls_hp ?? 0) < (city.walls_max_hp ?? 0) ? `${city.name}'s Walls are damaged and never regrow. This build restores them.` : `Upgrade Walls adds ${WALLS_HP_PER_LEVEL} to ${city.name}'s Walls.`;
    if (item.kind === 'Building') {
      const yields = buildingStatsFor(factionOf(view), item.building).yields;
      const gain = (['food', 'production', 'gold', 'science'] as YieldKey[]).find((key) => (yields[key] ?? 0) > 0);
      if (gain) return `${itemName(item, factionOf(view))} adds ${yields[gain]} ${YIELD_LABEL[gain]} in ${city.name}.`;
      return `${itemName(item, factionOf(view))} helps ${city.name}.`;
    }
    return `${city.name} can put its Production to work.`;
  }
  if (order.type === 'SetResearch' || order.type === 'SetResearchQueue') {
    return researchWhy(order, view);
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
  const name = item && (item.kind === 'Building' && item.building === 'UpgradeWalls' ? wallsUpgradeName(city.walls_level ?? Number(city.walls), itemName({ kind: 'Building', building: 'Walls' }, factionOf(ctx.view)), wallsRepairing(city, ctx.view.players.find((p) => p.id === city.owner)?.era ?? 1)) : itemName(item, factionOf(ctx.view)));
  return { id: `city:${city.id}`, title: name ? `Build ${name} in ${city.name}` : `Choose a build in ${city.name}`,
    why: order ? advisorWhy(order, ctx.view, city) : `${city.name} needs a build to use its Production.`,
    current: selected(ctx, 'city', city.id) ? 1 : 0,
    steps: [
      { instruction: `${ctx.touch ? 'Tap' : 'Click'} ${city.name}.`, target: tile(city.pos) },
      { instruction: name ? `${ctx.touch ? 'Tap' : 'Click'} Build on ${name}.` : 'Choose a unit or building to build.', target: item ? control('build', itemKey(item)) : null },
    ] };
}
function researchSuggestion(ctx: AdvisorContext, order?: Extract<Order, { type: 'SetResearch' | 'SetResearchQueue' }>): Suggestion {
  const planned = order?.type === 'SetResearch' ? [order.tech] : order?.techs ?? [];
  const available = new Set(ctx.view.research.available.map((option) => option.tech));
  const techs = planned.filter((tech) => available.has(tech)).slice(0, 3);
  const queue = effectiveResearchQueue(validOrders(ctx), ctx.view.research);
  const matched = techs.findIndex((tech, i) => queue[i] !== tech);
  const done = matched < 0 ? techs.length : matched;
  const names = techs.map((tech) => TECH_NAME[tech]);
  const title = names.length > 1 ? `Queue research: ${names.join(', then ')}` : names.length ? `Research ${names[0]}` : 'Choose research';
  const steps: AdvisorStep[] = [
    { instruction: `${ctx.touch ? 'Tap' : 'Click'} Research at the top.`, key: ctx.touch ? undefined : 'T', target: control('research') },
    { instruction: names[0] ? `${ctx.touch ? 'Tap' : 'Click'} ${names[0]}.` : 'Choose a technology.', target: techs[0] ? control('tech', techs[0]) : null },
    ...techs.slice(1).map((tech): AdvisorStep => ({ instruction: ctx.touch
      ? `Tap This era, then + Queue on ${TECH_NAME[tech]}.`
      : `Shift-click ${TECH_NAME[tech]} in the tree, or click + Queue.`, target: control('queueTech', tech) })),
  ];
  return { id: 'research', title, why: order && techs.length ? researchWhy({ type: 'SetResearch', tech: techs[0] }, ctx.view) : RESEARCH_FALLBACK,
    current: ctx.techOpen ? 1 + done : 0, steps };
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

/** The early lesson on winning: the target, the hold, the last turn and the four ways to score. */
export function howToWin(v: PlayerView): Suggestion {
  const mine = v.standings.find((s) => s.team === v.team);
  const target = mine?.target ?? 10, team = (mine?.members.length ?? 1) > 1 ? ' with your team' : '';
  return { id: 'howToWin', title: 'How to win', current: 0, acknowledge: true,
    why: `Get ${target} victory points (VP)${team} and keep them for ${HOLD_TURNS} turns. If nobody does by turn ${effectiveTurnLimit(v.turn_limit)}, the most VP then wins.`,
    steps: [{ instruction: 'Score VP with Age cards, rival capitals, world wonders and your faction goal. The Victory race shows the score.', target: control('race') }] };
}

export function advisorSuggestion(ctx: AdvisorContext, seen: readonly string[], skipped: readonly string[], practice: Practice = {}, tips: TipLog = {}, locks: LockLog = {}): Suggestion | null {
  const v = ctx.view;
  if (v.finished || v.players.find((p) => p.id === v.me)?.alive === false) return null;
  if (v.turn === 1 && !seen.includes('welcome')) return { id: 'welcome', title: 'Your advisor will guide you', why: "I'll suggest what to do each turn and show you where to click.", steps: [], current: 0, acknowledge: true };
  if (v.turn > 1 && seen.includes('welcome') && !seen.includes('report')) return { id: 'report', title: 'Read the Turn report', why: 'The Turn report lists battles, discoveries and failed orders.', steps: [{ instruction: 'Press L to open it.', key: ctx.touch ? undefined : 'L', target: null }], current: 0, acknowledge: true };
  // How to win, early: before the first score check, so a new player knows what VP are for.
  if (v.turn >= 3 && seen.includes('report') && !seen.includes('howToWin')) return howToWin(v);
  const plan = ctx.plan === undefined ? advisorOrders(v) : ctx.plan;
  const orders = validOrders(ctx);
  const decisions = pendingDecisions(v, orders, ctx.errors ?? [], ctx.local);
  const choice = plan?.find((o): o is Extract<Order, { type: 'SetResearch' | 'SetResearchQueue' }> => o.type === 'SetResearch' || o.type === 'SetResearchQueue');
  const queue = effectiveResearchQueue(orders, v.research);
  const nextTech = v.research.available.find((option) => !queue.includes(option.tech))?.tech;
  if (!seen.includes('researchQueue') && !skipped.includes('researchQueue') &&
      nextTech &&
      ((ctx.techOpen && queue.length === 1) || (ctx.researchQueueDrained && queue.length === 0))) {
    return { id: 'researchQueue', title: 'Queue your research', acknowledge: true, current: ctx.techOpen ? 1 : 0,
      why: queue.length ? 'Queue more technologies to keep researching after the current one.' : 'Choose a technology to resume research.',
      steps: [{ instruction: `${ctx.touch ? 'Tap' : 'Click'} Research at the top.`, target: control('research') },
        { instruction: ctx.touch ? `Tap This era, then + Queue on ${TECH_NAME[nextTech]}.`
          : `Shift-click ${TECH_NAME[nextTech]} in the tree, or click + Queue on ${TECH_NAME[nextTech]}.`, target: control('queueTech', nextTech) }] };
  }
  const pendingQueue = pendingResearchQueue(orders);
  if (pendingQueue?.length && choice?.type === 'SetResearchQueue' && !skipped.includes('research')) {
    const recommended = choice.techs.filter((tech) => v.research.available.some((option) => option.tech === tech)).slice(0, 3);
    if (pendingQueue.length < recommended.length && pendingQueue.every((tech, i) => recommended[i] === tech)) return researchSuggestion(ctx, choice);
  }
  // Research is chosen but its panel is still open: closing it is the next step, since the rest of the turn is behind it.
  // Only after a choice made this turn, so opening Research later to browse isn't nagged; "Not now" dismisses it.
  const chosen = pendingResearchQueue(orders);
  if (ctx.techOpen && chosen?.length && !skipped.includes('closeResearch') && !learned(practice, 'closeResearch')) {
    const tech = TECH_NAME[chosen[0]];
    return { id: 'closeResearch', title: 'Close Research', why: tech ? `${tech} is chosen. Your other decisions are on the map.` : 'Your research is chosen. Your other decisions are on the map.',
      steps: [{ instruction: `${ctx.touch ? 'Tap' : 'Click'} Close.`, key: ctx.touch ? undefined : 'Esc', target: control('closeResearch') }], current: 0 };
  }
  const handled = (kind: string) => kind === 'city' || kind === 'unit' || kind === 'research';
  const firstHandled = decisions.findIndex((d) => handled(d.kind));
  const firstUnanswered = decisions.findIndex((d, i) => !handled(d.kind) && (firstHandled < 0 || i < firstHandled) && !skipped.includes(`answer:${d.kind}:${'id' in d ? d.id : ''}`));
  if (firstUnanswered >= 0) {
    const d = decisions[firstUnanswered], id = `answer:${d.kind}:${'id' in d ? d.id : ''}`;
    const peace = d.kind === 'diplomacy' && v.diplomacy.proposals.some((p) => p.id === d.id && (p.kind ?? 'Peace') === 'Peace');
    return { id, title: peace ? 'Answer the peace talks' : 'Answer this first', why: d.label, current: 0,
      steps: [{ instruction: `${ctx.touch ? 'Tap' : 'Click'} ${ctx.turnLabel ?? d.label}.`, target: control('turn') }] };
  }
  const garrison = ownUnits(v).find((u) => u.occupation_lock != null);
  if (garrison && !seen.includes('garrisonLock') && !ctx.techOpen && !ctx.selection) {
    const city = occupiedCityName(v, garrison), people = ownerName(v, garrison.occupation_lock!);
    return { id: 'garrisonLock', title: `Your ${unitName(v, garrison)} holds ${city}`,
      why: `Your military units in ${city} stay as its garrison until Peace with ${people} or their elimination. ${city} builds nothing until then.`,
      steps: [{ instruction: `Make Peace with ${people} in Diplomacy to free your garrison.`, target: control('diplomacy') }],
      current: 0, acknowledge: true };
  }
  // Situational advice comes before the routine decisions, but never interrupts a unit or panel already in hand.
  if (seen.includes('welcome') && !ctx.techOpen) {
    const tip = strategyTip(v, tips, ctx.touch, ctx.selection, validOrders(ctx), ctx.placingTile ?? null, locks);
    if (tip && !skipped.includes(tip.id) && (!ctx.selection || tip.current > 0)) return tip;
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
      return researchSuggestion(ctx, choice);
    }
    if (d.kind === 'unit') {
      if (skipped.includes(`unit:${d.id}`)) continue;
      const unit = ownUnits(v).find((u) => u.id === d.id);
      if (!unit) continue;
      const choice = plan?.find((o) => 'unit' in o && o.unit === unit.id);
      if (learned(practice, choice ? practiceKind(choice, v) : unit.kind === 'Settler' ? null : 'move')) continue;
      const suggestion = unitSuggestion(ctx, unit, choice) ?? unitSuggestion(ctx, unit);
      if (suggestion) return suggestion;
    }
  }
  if (decisions.length || learned(practice, 'endTurn')) return null;
  const solo = ctx.humans <= 1 && !ctx.remote; // Matches the turn button's label (SeatBar).
  return { id: 'endTurn', title: solo ? 'End the turn' : 'Ready for the turn', why: 'Your orders are set. Every empire’s orders happen together.',
    steps: [{ instruction: `${ctx.touch ? 'Tap' : 'Click'} ${solo ? 'End turn' : 'Ready'}.`, key: ctx.touch ? undefined : 'Return', target: control('turn', 'end') }], current: 0 };
}

interface AdvisorPreferences { enabled: boolean; seen: string[]; practice: Practice; tips: TipLog; locks: LockLog }
export function readAdvisorPreferences(): AdvisorPreferences {
  try {
    const raw: unknown = JSON.parse(globalThis.localStorage?.getItem(ADVISOR_KEY) ?? 'null');
    if (raw && typeof raw === 'object') {
      const p = raw as { enabled?: unknown; seen?: unknown; tutorial?: unknown; practice?: unknown };
      return { enabled: p.enabled !== false, practice: readPractice(p.practice), tips: readTips((p as { tips?: unknown }).tips), locks: readTips((p as { locks?: unknown }).locks), seen: Array.isArray(p.seen) ? p.seen.filter((x): x is string => typeof x === 'string') : Array.isArray(p.tutorial) ? p.tutorial.filter((x): x is string => typeof x === 'string') : [] };
    }
  } catch { /* Storage may be unavailable. */ }
  return { enabled: true, seen: [], practice: {}, tips: {}, locks: {} };
}
const PRACTICE_KINDS: readonly PracticeKind[] = ['move', 'fortify', 'skip', 'explore', 'endTurn', 'closeResearch'];
function readPractice(raw: unknown): Practice {
  if (!raw || typeof raw !== 'object') return {};
  const out: Partial<Record<PracticeKind, number>> = {};
  for (const kind of PRACTICE_KINDS) {
    const n = (raw as Record<string, unknown>)[kind];
    if (typeof n === 'number' && Number.isFinite(n) && n > 0) out[kind] = Math.min(Math.floor(n), PRACTICE_TURNS);
  }
  return out;
}
function readTips(raw: unknown): TipLog {
  if (!raw || typeof raw !== 'object') return {};
  return Object.fromEntries(Object.entries(raw as Record<string, unknown>).filter((e): e is [string, number] => typeof e[1] === 'number' && Number.isFinite(e[1])));
}
function savePreferences({ enabled, seen, practice, tips, locks }: AdvisorPreferences) {
  try { globalThis.localStorage?.setItem(ADVISOR_KEY, JSON.stringify({ enabled, seen, practice, tips, locks })); } catch { /* Session state still works. */ }
}
interface AdvisorState extends AdvisorPreferences {
  skippedTurn: number;
  skipped: string[];
  markSeen(id: string): void;
  /** Turn-scoped, so a routine done twice in one turn counts once. */
  practisedTurn: number;
  practised: PracticeKind[];
  practise(kinds: readonly PracticeKind[], turn: number): void;
  resetPractice(): void;
  /** Record a tip as shown on this turn. An earlier turn than any logged means a new game: the log starts over. */
  noteTip(key: string, turn: number): void;
  observeLocks(view: PlayerView): void;
  skip(id: string, turn: number): void;
  setEnabled(enabled: boolean): void;
}
export const useAdvisor = create<AdvisorState>((set) => ({
  ...readAdvisorPreferences(), skippedTurn: -1, skipped: [], practisedTurn: -1, practised: [],
  practise: (kinds, turn) => set((s) => {
    const done = s.practisedTurn === turn ? s.practised : [];
    const fresh = [...new Set(kinds)].filter((k) => !done.includes(k));
    if (!fresh.length) return s;
    const practice = { ...s.practice };
    for (const k of fresh) practice[k] = Math.min((practice[k] ?? 0) + 1, PRACTICE_TURNS);
    const next = { ...s, practice, practisedTurn: turn, practised: [...done, ...fresh] };
    savePreferences(next);
    return next;
  }),
  markSeen: (id) => set((s) => { if (s.seen.includes(id)) return s; const next = { ...s, seen: [...s.seen, id] }; savePreferences(next); return next; }),
  skip: (id, turn) => set((s) => ({ skippedTurn: turn, skipped: [...(s.skippedTurn === turn ? s.skipped : []), id] })),
  noteTip: (key, turn) => set((s) => {
    if (s.tips[key] === turn) return s;
    const newGame = Object.entries(s.tips).some(([k, t]) => !k.startsWith('#') && t > turn);
    const log = newGame ? {} : s.tips, count = `#${tipKind(key)}`;
    const next = { ...s, tips: { ...log, [key]: turn, [count]: (log[count] ?? 0) + 1 } };
    savePreferences(next);
    return next;
  }),
  observeLocks: (view) => set((s) => {
    const locks: Record<string, number> = {};
    for (const unit of view.units) {
      if (unit.owner !== view.me || unit.occupation_lock == null) continue;
      const key = `${unit.id}:${unit.occupation_lock}`;
      const since = s.locks[key];
      locks[key] = since !== undefined && since <= view.turn ? since : view.turn;
    }
    if (Object.keys(locks).length === Object.keys(s.locks).length && Object.entries(locks).every(([key, since]) => s.locks[key] === since)) return s;
    const next = { ...s, locks }; savePreferences(next); return next;
  }),
  resetPractice: () => set((s) => { const next = { ...s, practice: {}, practised: [] }; savePreferences(next); return next; }),
  setEnabled: (enabled) => set((s) => { const next = { ...s, enabled }; savePreferences(next); return next; }),
}));
