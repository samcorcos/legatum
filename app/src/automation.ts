import type { CityBuildFocus, CityView, Faction, Order, PlayerView, ProductionItem, UnitView } from './types';
import { buildingStatsFor, itemName } from './data';
import { pendingUnitOrder } from './orders';
import { loadedWasmModule } from './wasmClient';

export const STOP_REASON = {
  Complete: 'nothing reachable left to explore',
  Threatened: 'known enemy military threatens this unit',
  Damaged: 'health below 60 HP',
  Blocked: 'the planned route was blocked',
};

/** A manual order overrides the persisted intent immediately in the UI. */
export function isExploring(unit: UnitView, orders: Order[]): boolean {
  const pending = pendingUnitOrder(orders, unit.id);
  return pending ? pending.type === 'SetAutomation' && pending.automation === 'Explore' : unit.automation === 'Explore';
}

export function automatedUnits(view: PlayerView, orders: Order[]): UnitView[] {
  return view.units.filter((u) => u.owner === view.me && isExploring(u, orders)).sort((a, b) => a.id - b.id);
}

/** Both local and online seats plan from exactly the view displayed in their client. */
export async function addAutomatedOrders(view: PlayerView, orders: Order[]): Promise<Order[]> {
  const units = automatedUnits(view, orders);
  if (!units.length) return orders;
  const { loadWasmModule } = await import('./wasmClient');
  const wasm = await loadWasmModule();
  const json = JSON.stringify(view);
  const plan = (_v: PlayerView, unit: number): Order | null => JSON.parse(wasm.explore_step(json, unit));
  const ids = new Set(units.map((u) => u.id));
  return [...orders.filter((o) => !('unit' in o && o.type !== 'HireCompany' && ids.has(o.unit))), ...units.map((u): Order =>
    plan(view, u.id) ?? { type: 'SetAutomation', unit: u.id, automation: null, stop_reason: 'Complete' })];
}

const generated = new WeakSet<Order>();
const previews = new WeakMap<PlayerView, Map<string, CityAutomationPlan>>();

export const AUTOMATION_NO_CHOICE = "Nothing here fits its automation focus. Choose this turn's build.";
export interface CityAutomationPlan { orders: Order[]; unchosen_cities: number[] }

/** Exact chooser preview when WASM is already loaded; Ready always loads it if needed. */
export function cityAutomationPreview(view: PlayerView, orders: Order[]): CityAutomationPlan | null {
  const wasm = loadedWasmModule();
  if (!wasm || !hasAutomatedCities(view, orders)) return null;
  const clean = orders.filter((o) => !generated.has(o) || o.type === 'SetCityAutomation');
  const orderJson = JSON.stringify(clean);
  const cached = previews.get(view)?.get(orderJson);
  if (cached) return cached;
  try {
    const plan = JSON.parse(wasm.city_automation_orders(JSON.stringify(view), orderJson)) as CityAutomationPlan;
    const byOrders = previews.get(view) ?? new Map<string, CityAutomationPlan>();
    byOrders.set(orderJson, plan);
    previews.set(view, byOrders);
    return plan;
  } catch {
    // Preview is optional; Ready reports a planning error if the real call fails.
    return null;
  }
}

export function cityAutomationFocus(city: number, view: PlayerView, orders: Order[]): CityBuildFocus | null {
  const pending = orders.findLast((o) => o.type === 'SetCityAutomation' && o.city === city);
  return pending?.type === 'SetCityAutomation' ? pending.focus : view.cities.find((c) => c.id === city)?.detail?.build_automation ?? null;
}

/** I turns a city's automation on with the focus last chosen; Shift+I steps through this order. */
const FOCUS_CYCLE: CityBuildFocus[] = ['Balanced', 'Military', 'Science', 'Gold'];
const FOCUS_KEY = 'legatum.lastAutomationFocus';
let lastFocus: CityBuildFocus | null = null;

export function lastAutomationFocus(): CityBuildFocus {
  if (!lastFocus) {
    try {
      const saved = localStorage.getItem(FOCUS_KEY) as CityBuildFocus | null;
      if (saved && FOCUS_CYCLE.includes(saved)) lastFocus = saved;
    } catch { /* storage blocked: Balanced */ }
  }
  return lastFocus ?? 'Balanced';
}

export function nextAutomationFocus(focus: CityBuildFocus | null): CityBuildFocus {
  return focus ? FOCUS_CYCLE[(FOCUS_CYCLE.indexOf(focus) + 1) % FOCUS_CYCLE.length] : lastAutomationFocus();
}

/** A player's edit supersedes only a previously generated choice, never their own queue. */
export function setCityAutomation(orders: Order[], city: number, focus: CityBuildFocus | null): Order[] {
  if (focus) {
    lastFocus = focus;
    try { localStorage.setItem(FOCUS_KEY, focus); } catch { /* per-viewer convenience only */ }
  }
  return [...orders.filter((o) => !(o.type === 'SetCityAutomation' && o.city === city)
    && !(generated.has(o) && 'city' in o && o.city === city)), { type: 'SetCityAutomation', city, focus }];
}

export function hasAutomatedCities(view: PlayerView, orders: Order[]): boolean {
  return view.cities.some((c) => c.owner === view.me && !!cityAutomationFocus(c.id, view, orders));
}

/** Plans from the owner-only view supplied by the client; no hidden GameState reaches WASM. */
export async function addCityAutomatedOrders(view: PlayerView, orders: Order[]): Promise<CityAutomationPlan> {
  if (!hasAutomatedCities(view, orders)) return { orders, unchosen_cities: [] };
  const { loadWasmModule } = await import('./wasmClient');
  const wasm = await loadWasmModule();
  const clean = orders.filter((o) => !generated.has(o) || o.type === 'SetCityAutomation');
  const { orders: planned, unchosen_cities } = JSON.parse(wasm.city_automation_orders(JSON.stringify(view), JSON.stringify(clean))) as CityAutomationPlan;
  if (!planned.length) return { orders: clean, unchosen_cities };
  const picked = new Set(planned.filter((o) => o.type === 'SetCityAutomation').map((o) => (o as Extract<Order, { type: 'SetCityAutomation' }>).city));
  const keep = clean.filter((o) => !(o.type === 'SetCityAutomation' && picked.has(o.city)));
  planned.forEach((o) => generated.add(o));
  return { orders: [...keep, ...planned], unchosen_cities };
}

const isFocusBuilding = (focus: CityBuildFocus, item: ProductionItem, faction: Faction): boolean => {
  if (item.kind !== 'Building') return false;
  const yields = buildingStatsFor(faction, item.building).yields;
  return focus === 'Science' ? (yields.science ?? 0) > 0 : focus === 'Gold' && (yields.gold ?? 0) > 0;
};
const isArmyUnit = (item: ProductionItem): boolean => item.kind === 'Unit' && item.unit !== 'Scout' && item.unit !== 'Settler' && item.unit !== 'HeirToTheThrone';

/** Why automation chose `pick` for `city`: the real reason, never a stock phrase. */
export function autoPickReason(pick: ProductionItem, focus: CityBuildFocus, city: CityView, view: PlayerView): string {
  const faction = view.players.find((p) => p.id === city.owner)?.faction ?? 'Switzerland';
  const name = itemName(pick, faction);
  const aName = `${/^[AEIOU]/.test(name) ? 'an' : 'a'} ${name}`;
  if (focus === 'Balanced') return `${name}: Balanced picks what ${city.name} needs most.`;
  if (isFocusBuilding(focus, pick, faction)) return `${name}: ${focus} focus, ${focus === 'Science' ? 'the best research building available' : 'the best economy building available'}.`;
  if (focus === 'Military' && isArmyUnit(pick)) return `${name}: Military focus, the best military unit available.`;
  const focusBuildAvailable = (city.detail?.options ?? []).some((o) => !o.resource_blocker && (focus === 'Military' ? isArmyUnit(o.item) : isFocusBuilding(focus, o.item, faction)));
  const none = `No ${focus} ${focus === 'Military' ? 'unit' : 'building'} available`;
  // A focus always builds its own kind when it can; the one exception left is a unit whose upkeep Gold can't cover.
  if (!focusBuildAvailable) return `${none}; ${aName} is the next best build.`;
  return focus === 'Military' ? `${name}: Not enough Gold to pay another unit's upkeep, so it builds the next best thing.` : `${name}: no ${focus} building can start now, so this is the next best build.`;
}
