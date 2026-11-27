/** One-time lessons for useful optional controls, independent of the Hard AI's chosen orders. */
import { cityAutomationFocus, isExploring } from '../automation';
import { itemKey, itemName, SIEGE_USES_IRON } from '../data';
import { reconBlocker } from '../airWing';
import { upgradeCheck } from './UnitImprovements';
import { pendingSpend } from './productionPurchases';
import { isThreat } from '../diplomacy';
import { distance, hexEq } from '../hex';
import type { Order } from '../types';
import type { AdvisorContext, AdvisorStep, Practice, Suggestion } from './advisorModel';

export const DISCOVERY_IDS = ['cityAutomation', 'autoExplore', 'tileImprovements', 'productionQueue', 'heal', 'autoFire', 'modernize', 'airOperations', 'teamPing', 'siege', 'projects', 'mapYields', 'cities'] as const;
export type DiscoveryId = typeof DISCOVERY_IDS[number];

/** Using a feature is evidence it is known, even if its lesson was never displayed. */
export function discoveriesUsed(orders: readonly Order[]): DiscoveryId[] {
  const used = new Set<DiscoveryId>();
  const automaticCities = new Set(orders.filter((o) => o.type === 'SetCityAutomation' && o.picked != null).map((o) => (o as Extract<Order, { type: 'SetCityAutomation' }>).city));
  for (const order of orders) {
    if (order.type === 'SetCityAutomation' && order.focus) used.add('cityAutomation');
    if (order.type === 'SetAutomation' && order.automation === 'Explore') used.add('autoExplore');
    if (order.type === 'UpgradeSiege') used.add('siege');
    if (order.type === 'SetProduction' && order.item.kind === 'Project') used.add('projects');
    if (order.type === 'UpgradeUnit') used.add('modernize');
    if (order.type === 'AirStrike' || order.type === 'AirRecon' || order.type === 'Rebase') used.add('airOperations');
    if (order.type === 'SetAutoFire' && order.enabled) used.add('autoFire');
    if (order.type === 'SleepUnit' && order.until_healed) used.add('heal');
    if (order.type === 'SetProductionQueue' && !automaticCities.has(order.city) && order.queue.length > 1) used.add('productionQueue');
    if ((order.type === 'SetProduction' && !automaticCities.has(order.city) && order.item.kind === 'Improvement')
      || (order.type === 'SetProductionQueue' && !automaticCities.has(order.city) && order.queue.some((item) => item.kind === 'Improvement'))) used.add('tileImprovements');
  }
  return [...used];
}

export function discoverySuggestion(ctx: AdvisorContext, seen: readonly string[], skipped: readonly string[], practice: Practice): Suggestion | null {
  const v = ctx.view;
  if (ctx.discoveryTurn === v.turn || v.finished || ctx.techOpen || ctx.placingTile != null || (ctx.mode && ctx.mode !== 'normal')) return null;
  const used = discoveriesUsed(ctx.orders);
  // A newly used feature ends optional teaching for this turn, before the React persistence effect runs.
  if (used.some((id) => !seen.includes(id))) return null;
  const known = new Set([...seen, ...skipped, ...used, ...(ctx.yieldsVisible ? ['mapYields'] : [])]);
  const available = (id: DiscoveryId) => !known.has(id);
  const faction = v.players.find((p) => p.id === v.me)?.faction;
  const click = ctx.touch ? 'Tap' : 'Click';
  const control = (name: Extract<NonNullable<AdvisorStep['target']>, { kind: 'control' }>['name'], value?: string): AdvisorStep['target'] => ({ kind: 'control', name, value });
  const lesson = (id: DiscoveryId, title: string, why: string, steps: AdvisorStep[], current: number): Suggestion =>
    ({ id, title, why, steps, current, acknowledge: true });
  const safe = (pos: { q: number; r: number }) => !v.units.some((u) => u.military && isThreat(v, u.owner) && distance(pos, u.pos) <= 6);
  const cities = v.cities.filter((c) => c.owner === v.me);
  const cityOrder = (id: number) => ctx.orders.some((o) => 'city' in o && o.city === id && (o.type === 'SetProduction' || o.type === 'SetProductionQueue'));
  // After the opening, offer delegation when city management becomes repetitive. A large,
  // mature empire reaches this sooner; a late-era small empire can benefit too.
  if (available('cityAutomation') && ((v.turn >= 80 && cities.length >= 3) || v.research.era >= 4)) {
    const city = cities.find((c) => c.population >= 4 && c.detail && c.detail.resistance === 0 && c.occupation_lock == null
      && !cityAutomationFocus(c.id, v, ctx.orders) && !cityOrder(c.id) && safe(c.pos)
      && (!ctx.selection || ctx.selection.kind === 'city' && ctx.selection.id === c.id));
    if (city) return lesson('cityAutomation', `Let ${city.name} manage its economy`,
      'Automate chooses economic buildings and tile improvements, favoring Gold, then Science and Production. Your queue goes first.',
      [{ instruction: `${click} ${city.name}.`, target: { kind: 'tile', hex: city.pos } },
        { instruction: ctx.touch ? 'Tap Automate. Tap it again to take control.' : 'Click Automate. Press I again to take control.', key: ctx.touch ? undefined : 'I', target: control('automate') }],
      ctx.selection?.kind === 'city' && ctx.selection.id === city.id ? 1 : 0);
  }
  const city = ctx.selection?.kind === 'city' ? cities.find((c) => c.id === ctx.selection!.id) : undefined;
  if (city?.detail && city.detail.resistance === 0 && city.occupation_lock == null && !cityAutomationFocus(city.id, v, ctx.orders) && !cityOrder(city.id) && safe(city.pos)) {
    const d = city.detail;
    const improvement = d.options.find((o) => o.item.kind === 'Improvement' && o.turns >= 0 && !o.resource_blocker);
    if (available('tileImprovements') && !d.producing && improvement) return lesson('tileImprovements', 'Improve your resource tiles',
      'Resource tiles gain their extra yields when you improve them. Improvements upgrade with technology.',
      [{ instruction: `${click} Build ${itemName(improvement.item, faction)}.`, target: control('build', itemKey(improvement.item)) }], 0);
    const goldProject = d.options.find((o) => o.item.kind === 'Project' && o.item.project === 'Gold');
    const economicBuild = d.options.some((o) => (o.item.kind === 'Building' || o.item.kind === 'Wonder' || o.item.kind === 'Improvement') && !o.resource_blocker && o.turns >= 0);
    if (available('projects') && !d.producing && !economicBuild && goldProject) return lesson('projects', 'Keep an idle city earning',
      'Projects earn a yield every turn instead of finishing a building. They keep running until you choose something else.',
      [{ instruction: `${click} Run ${itemName(goldProject.item, faction)}.`, target: control('build', itemKey(goldProject.item)) }], 0);
    const next = d.options.find((o) => o.turns >= 0 && !o.resource_blocker && itemKey(o.item) !== (d.producing ? itemKey(d.producing) : ''));
    if (available('productionQueue') && d.producing && d.producing.kind !== 'Project' && (d.production_queue?.length ?? 0) <= 1 && next) return lesson('productionQueue', 'Plan the next build',
      'Queue a build to start it automatically after the current one finishes.',
      [{ instruction: `${click} + Queue beside ${itemName(next.item, faction)}.`, target: control('queueBuild', itemKey(next.item)) }], 0);
  }
  if (city?.detail && city.detail.resistance === 0 && city.occupation_lock == null && !cityAutomationFocus(city.id, v, ctx.orders) && !cityOrder(city.id) && v.turn >= 12 && available('mapYields') && safe(city.pos)) return lesson('mapYields', 'See what each tile yields',
    'The Yields overlay shows Food, Production, Gold and Science directly on explored tiles.',
    [{ instruction: `${click} Lens, then turn on Yields.`, target: control('mapYields') }], 0);
  if (!ctx.selection && !ctx.touch && cities.length >= 3 && v.turn >= 20 && available('cities')) return lesson('cities', 'Manage your cities together',
    'Cities compares population, income and builds. Select several cities there to automate them together.',
    [{ instruction: 'Click Cities at the top.', key: 'C', target: control('cities') }], 0);
  if (!ctx.selection && ctx.canPing && v.turn >= 3 && available('teamPing')) return lesson('teamPing', 'Mark a place for your teammate',
    'Ping marks a tile for your human teammates until the turn ends.',
    [{ instruction: `${click} Ping, then a tile on the map.`, key: ctx.touch ? undefined : 'P', target: control('ping') }], 0);
  if (ctx.selection?.kind === 'air' && available('airOperations')) {
    const plane = v.aircraft?.find((p) => p.id === ctx.selection!.id && p.owner === v.me);
    const base = plane && cities.find((c) => c.id === plane.city);
    if (plane && base && !ctx.orders.some((o) => 'plane' in o && o.plane === plane.id) && !reconBlocker(v, base, plane, ctx.orders)) {
      return lesson('airOperations', 'Use your aircraft',
        'Aircraft can scout, strike or move between city hangars. Each aircraft gets one order per turn.',
        [{ instruction: `${click} Recon… to choose a tile to scout.`, key: ctx.touch ? undefined : 'A', target: control('airRecon') }], 0);
    }
  }
  const unit = ctx.selection?.kind === 'unit' ? v.units.find((u) => u.owner === v.me && u.id === ctx.selection!.id)
    : !ctx.selection ? v.units.find((u) => u.owner === v.me && u.kind === 'Scout' && !u.goto && !u.standing_order && !isExploring(u, ctx.orders)) : undefined;
  if (!unit || ctx.orders.some((o) => 'unit' in o && o.unit === unit.id)) return null;
  const selected = ctx.selection?.kind === 'unit' && ctx.selection.id === unit.id;
  const unitSteps = (instruction: string, name: 'explore' | 'heal' | 'autoFire', key: string): AdvisorStep[] => [
    { instruction: `${click} your ${itemName({ kind: 'Unit', unit: unit.kind }, faction)}.`, target: { kind: 'tile', hex: unit.pos } },
    { instruction: `${click} ${instruction}.`, key: ctx.touch ? undefined : key, target: control(name) },
  ];
  if (selected && available('modernize')) {
    const balance = pendingSpend(v, ctx.orders, ctx.errors ?? []).available({ type: 'UpgradeUnit', unit: unit.id }, ctx.orders.length);
    const upgrade = upgradeCheck(unit, v, balance.gold, balance.supply);
    if (upgrade && !upgrade.blocker && upgrade.cost != null) return lesson('modernize', 'Upgrade your experienced units',
      `Upgrade this unit to ${itemName({ kind: 'Unit', unit: upgrade.to }, faction)} for ${upgrade.cost} Gold. It keeps its promotions.`,
      [{ instruction: `${click} Upgrade to ${itemName({ kind: 'Unit', unit: upgrade.to }, faction)}.`, key: ctx.touch ? undefined : 'U', target: control('upgrade') }], 0);
  }
  if (selected && available('siege') && unit.siege_offer && !unit.siege_offer.blocker) {
    const balance = pendingSpend(v, ctx.orders, ctx.errors ?? []).available({ type: 'UpgradeSiege', unit: unit.id }, ctx.orders.length);
    const iron = (v.supply?.find((s) => s.resource === 'Iron')?.spare ?? 0) - (balance.supply.get('Iron') ?? 0);
    if (balance.gold >= unit.siege_offer.price.total && (!SIEGE_USES_IRON || iron > 0)) return lesson('siege', 'Equip a unit to attack Walls',
      `Siege helps this unit attack city Walls. The equipment costs ${unit.siege_offer.price.total} Gold.`,
      [{ instruction: `${click} Siege under Improve.`, target: control('siege') }], 0);
  }
  if (selected && available('heal') && unit.hp < 100 && (unit.resting_heal?.total ?? 0) > 0 && unit.standing_order !== 'Heal' && safe(unit.pos)) {
    return lesson('heal', 'Rest until healed', 'Heal rests this unit until it reaches full health, then asks for orders again.', unitSteps('Heal', 'heal', 'H'), 1);
  }
  if (selected && available('autoFire') && unit.ranged > 0 && unit.standing_order !== 'AutoFire'
    && cities.some((c) => hexEq(c.pos, unit.pos))) return lesson('autoFire', 'Let this unit guard its city',
    'Auto-fire holds this city and shoots enemies in range each turn. It never moves or declares War.', unitSteps('Auto-fire from city', 'autoFire', 'V'), 1);
  if (available('autoExplore') && !(practice.explore && practice.explore > 0) && v.turn >= 5 && unit.kind === 'Scout'
    && unit.hp >= 80 && unit.occupation_lock == null && !unit.goto && !unit.standing_order && !isExploring(unit, ctx.orders)
    && safe(unit.pos) && v.tiles.some((t) => !t.explored)) return lesson('autoExplore', 'Let your Scout explore',
    'Auto-explore finds reachable land and ruins while avoiding known threats. Give a manual order to take control.', unitSteps('Auto-explore', 'explore', 'E'), selected ? 1 : 0);
  return null;
}
