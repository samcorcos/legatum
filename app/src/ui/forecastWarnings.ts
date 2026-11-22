import type { BattleForecast, CityAssaultForecast, CityDetail, CityView, Faction, Hex, Order, PlayerView, UnitKind, UnitView } from '../types';
import { damage } from '../combat';
import { attacksInMelee, BUILT_IN_SIEGE_NOTE, isGreatWork, itemName, sameItem } from '../data';
import { wonderLostDecision, wonderRaceWarning } from '../wonderRace';
import { Game } from '../wasm-pkg/legatum.js';
import { effectiveProduction } from '../orders';
import { atWar, isThreat } from '../diplomacy';
import { distance, hexEq } from '../hex';
import { formatBreakdown } from '../format';
import { ownerName } from '../barbarians';
import { productionStallText } from '../cityProgress';
import { AUTOMATION_NO_CHOICE, cityAutomationFocus, cityAutomationPreview } from '../automation';
import { tombOnAfterOrders } from '../goalCities';

export { productionStallText };

/** A visible rival can take a civilian before a more distant capture order arrives. */
export function civilianCaptureRace(view: PlayerView, mover: UnitView, target: Hex, steps: number): string | null {
  if (steps < 2 || !mover.military) return null;
  const civilian = view.units.find((u) => !u.military && hexEq(u.pos, target) && atWar(view, u.owner));
  if (!civilian) return null;
  const canCapture = (owner: number) => {
    if (owner === civilian.owner) return false;
    if (owner === 255 || civilian.owner === 255) return true;
    const a = view.players.find((p) => p.id === owner)?.team;
    const b = view.players.find((p) => p.id === civilian.owner)?.team;
    return a !== undefined && b !== undefined && a !== b && view.diplomacy.relations.some((r) =>
      ((r.a === a && r.b === b) || (r.a === b && r.b === a)) && r.state === 'War');
  };
  const rivals = view.units.filter((u) => u.military && u.id !== mover.id && canCapture(u.owner) && distance(u.pos, target) === 1);
  if (!rivals.length) return null;
  const labels = rivals.map((u) => `${u.owner === 255 ? 'barbarian ' : ''}${itemName({ kind: 'Unit', unit: u.kind }, view.players.find((p) => p.id === u.owner)?.faction)}`);
  return `Reaches the ${civilian.kind} in ${steps} steps: ${labels.join(' and ')} ${rivals.length === 1 ? 'is' : 'are'} already next to it and may capture it first.`;
}

/** Occupation lock: warns before a military unit enters a captured city, or captures one, that it will stay as its garrison. */
export function occupationLockWarning(view: PlayerView, mover: UnitView | undefined, city: CityView | undefined): string | null {
  if (!mover?.military || !city || city.occupation_lock == null || hexEq(mover.pos, city.pos)) return null;
  return `Occupation: a unit that ends its move in ${city.name} stays as its garrison, holding the city down until Peace with ${ownerName(view, city.occupation_lock)}, and the city builds and buys nothing until then.`;
}

/** Occupation lock: an own city frozen until Peace, in the words core refuses its purchases with (`occupation::city_refusal`). Null when the city builds and buys. */
export function cityOccupiedLine(view: PlayerView, city: CityView): string | null {
  if (city.owner !== view.me || city.occupation_lock == null) return null;
  return `Occupied: ${city.name} builds and buys nothing until Peace with ${ownerName(view, city.occupation_lock)}`;
}

/** Occupation lock: the name of the city a locked unit holds. */
export function occupiedCityName(view: PlayerView, unit: UnitView): string {
  return view.cities.find((c) => hexEq(c.pos, unit.pos))?.name ?? 'this city';
}

/** Occupation lock: why a locked unit cannot leave, as core words its refusal (`occupation::refusal`); the refused click says this. Null when the unit is free. */
export function occupationRefusal(view: PlayerView, unit: UnitView): string | null {
  if (unit.occupation_lock == null) return null;
  const name = itemName({ kind: 'Unit', unit: unit.kind }, view.players.find((p) => p.id === unit.owner)?.faction);
  return `${name} is holding ${occupiedCityName(view, unit)} down and cannot leave until Peace with ${ownerName(view, unit.occupation_lock)}`;
}

/** Occupation lock: the unit card's lead line, the map's lock mark and the tile card say the same thing: the unit is the city's garrison. Null when the unit is free. */
export function occupationLead(view: PlayerView, unit: UnitView): string | null {
  if (unit.occupation_lock == null) return null;
  return `Garrisoning ${occupiedCityName(view, unit)}: holding the city down until Peace with ${ownerName(view, unit.occupation_lock)}.`;
}

/** The muted line under the lead: the reason, and what the lock costs. */
export function occupationNote(view: PlayerView, unit: UnitView): string | null {
  if (unit.occupation_lock == null) return null;
  return `${occupiedCityName(view, unit)}'s people will not accept your rule while ${ownerName(view, unit.occupation_lock)} still fights. This unit cannot leave or disband.`;
}

/** The tile card's line for a held unit. */
export function occupationTileLine(view: PlayerView, unit: UnitView): string | null {
  if (unit.occupation_lock == null) return null;
  return `Garrison of ${occupiedCityName(view, unit)} · held here until Peace with ${ownerName(view, unit.occupation_lock)}`;
}

/**
 * The forecast card's warning when the attacker could be destroyed (G18): any chance of losing
 * it, including a nonzero chance that rounds down to 0%.
 */
export function attackerRisk(f: BattleForecast, unitName: string): string | null {
  if (f.ranged) return null;
  const worst = damage(f.defender_strength.total, f.attacker_strength.total, 120);
  if (f.loss_chance <= 0 && worst < f.attacker_hp) return null;
  return `Risky: your ${unitName} may be destroyed (${f.loss_chance > 0 ? `${f.loss_chance}%` : 'under 1%'} chance).`;
}

/**
 * Walled cities fire back: the attacked city's shot at the attacker this turn, at a 100% roll, with
 * both strengths broken down. Null when the city can't fire at it (no Walls, out of range, 0 HP,
 * rule off).
 */
export function cityStrikeWarning(f: BattleForecast, cityName: string, unitName: string, fire?: number): { text: string; details: string; deadly: boolean } | null {
  const cs = f.city_strike;
  if (!cs) return null;
  const deadly = damage(cs.city_strength.total, cs.attacker_defense.total, 120) >= f.attacker_hp;
  return {
    text: `${cityName} will fire back: about −${fire ?? Math.min(cs.damage, f.attacker_hp)} HP to your ${unitName}${deadly ? ', and it could destroy it' : ''}. It fires at the weakest unit in range.`,
    details: `City ${formatBreakdown(cs.city_strength)} vs your defence ${formatBreakdown(cs.attacker_defense)}; rolls 80–120%, before movement, separate from the attack above.`,
    deadly,
  };
}

/** What the attack really costs once a walled city's shot is counted (playtest-6 H1, pillar 5). */
export interface FireOutcome {
  /** Mean HP the city's shot takes, over its 80-120% rolls. */
  fire: number;
  /** Mean HP lost in the attack itself, after the shot (0 for a ranged strike). */
  attack: number;
  /** Mean HP the attacker ends the turn with. */
  hpAfter: number;
  win: number;
  loss: number;
  /** Mean damage dealt to the defender. */
  dealt: number;
  /** Any outcome destroys the attacker, even when the chance rounds down to 0%. */
  canDie: boolean;
}

/** Sparta line infantry ignore wounds (core `combat::unit_attack_strength_for`). */

/**
 * The forecast with the attacked city's shot folded in. The city fires in the ranged phase, before
 * movement: a melee attacker then fights at its reduced HP (and wounded strength), so the odds are
 * recomputed over every city roll x attack roll x defence roll (41^3 outcomes, the core's integer
 * math). A ranged strike is planned from the same snapshot as the shot, so only its HP changes.
 * Null when the city does not fire.
 */
export function cityFireOutcome(f: BattleForecast, faction: Faction | undefined, kind: UnitKind): FireOutcome | null {
  if (!f.city_strike) return null;
  return JSON.parse(Game.combat_projection(JSON.stringify({ op: 'cityFireOutcome', forecast: f, faction, kind }))) as FireOutcome;
}

/**
 * Whom the city's shot goes to: the visible hostile military unit within two tiles with the lowest
 * (HP, id), as resolution picks it (core `forecast::city_strike_target`). Only your team's units are
 * counted: other empires' wars are not public, so a weaker third-party unit may still draw the shot.
 */
export function cityShotTarget(view: PlayerView, city: CityView): UnitView | null {
  if (!atWar(view, city.owner)) return null;
  const team = new Set(view.players.filter((p) => p.team === view.team).map((p) => p.id));
  return view.units
    .filter((u) => u.military && u.hp > 0 && team.has(u.owner) && distance(u.pos, city.pos) >= 1 && distance(u.pos, city.pos) <= 2)
    .sort((a, b) => a.hp - b.hp || a.id - b.id)[0] ?? null;
}

/**
 * The forecast headline for attacking a garrisoned city (core `combat::city_tile_defender`): who
 * takes the damage, and what a win does. Null when the target city has no visible garrison.
 */
export function garrisonHeadline(f: BattleForecast, view: PlayerView, cityName: string, ranged: boolean): string | null {
  const g = f.garrison;
  if (!g) return null;
  const name = itemName({ kind: 'Unit', unit: g.kind }, view.players.find((p) => p.id === g.owner)?.faction);
  if (g.shielded) {
    return `${cityName} absorbs damage first. The hit that empties its HP carries on to the ${name} garrison with the part of its damage the city did not absorb; later hits strike the garrison in full.`
      + (ranged ? ' Ranged fire cannot capture.' : ' A melee attacker captures if the garrison falls.');
  }
  return ranged
    ? `${cityName} has fallen: this shot hits its ${name} garrison (${g.hp} HP). Only a melee attack can capture the city.`
    : `${cityName} has fallen: you fight its ${name} garrison (${g.hp} HP), and destroying it captures the city.`;
}

/** One line of the combined card: a thing that takes damage, its HP before and after (all numbers come from core's `CityAssaultForecast`). */
export interface AssaultRow { key: 'city' | 'garrison'; label: string; hp: number; after: number; note?: string }

/**
 * The whole fight at an enemy city, in words (user, 2026-10-19, T187 Lugano): the city, the garrison under it and whether the
 * city is captured this turn, all from core's pooled forecast. Your unit's own HP result (city fire and counterattack included)
 * is the left side of the same card.
 */
export function assaultSummary(a: CityAssaultForecast, cityName: string, garrisonName: string | null): { title: string; rows: AssaultRow[]; capture: string; captureTone: 'good' | 'bad' | 'plain' } {
  // The carried hit rides on the city's own row: its HP before and after say what it absorbed, the note what reaches the garrison (core `AssaultGarrison::carried`).
  const g = a.garrison;
  const rows: AssaultRow[] = [{ key: 'city', label: cityName, hp: a.city_hp, after: a.city_hp_after, ...(g?.carried && g.carried > 0 ? { note: `carries −${g.carried} to the garrison` } : {}) }];
  if (g) {
    rows.push({ key: 'garrison', label: `${garrisonName ?? 'Garrison'} garrison`, hp: g.hp, after: g.hp_after, ...(g.destroyed_chance > 0 ? { note: g.destroyed_chance >= 100 ? 'destroyed' : `${g.destroyed_chance}% destroyed` } : {}) });
  }
  const left = g && g.hp_after > 0
    ? `its ${garrisonName ?? 'garrison'} survives with about ${g.hp_after} HP`
    : a.city_hp_after > 0 ? `${cityName} keeps about ${a.city_hp_after} HP` : '';
  let capture: string;
  let captureTone: 'good' | 'bad' | 'plain';
  if (!a.can_capture) {
    capture = `A shot cannot capture a city.${left ? ` After this turn ${left}.` : ''}`;
    captureTone = 'plain';
  } else if (a.capture_chance >= 100) {
    capture = `${cityName} is captured this turn.`;
    captureTone = 'good';
  } else if (a.capture_chance > 0) {
    capture = `${cityName} is captured this turn in ${a.capture_chance}% of outcomes; otherwise ${left || `${cityName} holds`}.`;
    captureTone = 'plain';
  } else {
    capture = `${cityName} is not captured this turn${left ? `: ${left}` : ''}.`;
    captureTone = 'bad';
  }
  const title = a.attacks > 1 ? `The whole fight at ${cityName}: ${a.attacks} attacks this turn` : `The whole fight at ${cityName}`;
  return { title, rows, capture, captureTone };
}

/**
 * What the city's Walls do to this attack (core `WallsForecast`): halve its damage to the city's HP, unless the attacking unit
 * has Siege. An Industrial or later unit has it built in and says so in core's words (`BUILT_IN_SIEGE_NOTE`). `siegeTier` is the viewer's own siege tier (core `data::siege_tier`), so the hint to buy Siege is shown only to a
 * player who knows Mathematics. Null when the target has no Walls.
 */
export function wallsLine(f: BattleForecast, cityName: string, unitLabel: string, siegeTier: number, dealt = 0): { text: string; tone: 'good' | 'plain' } | null {
  const w = f.walls;
  if (!w) return null;
  if (w.built_in) return { text: `Siege built in: your ${unitLabel} ${BUILT_IN_SIEGE_NOTE}.`, tone: 'good' };
  if (w.covered >= w.tier) {
    const ignored = ['', 'Walls', 'Walls and Castle', 'Walls, Castle and Star Fort'][w.tier];
    return { text: `Siege: your ${unitLabel} ignores ${cityName}'s ${ignored}.`, tone: 'good' };
  }
  if (w.covered > 0) {
    const defence = w.tier === 2 ? 'Castle' : 'Star Fort';
    const tech = w.tier === 2 ? 'Machinery' : 'Metallurgy';
    return { text: `Your Siege ignores Walls but not ${cityName}'s ${defence}: ${w.percent}% damage. ${tech} would reach it.`, tone: 'plain' };
  }
  const withSiege = w.percent > 0 && w.percent < 100 && dealt > 0 ? Math.round((dealt * 100) / w.percent) : 0;
  return {
    text: `Walls halve this attack on ${cityName}${siegeTier > 0 && withSiege > 0 ? `: ${dealt} HP, or ${withSiege} with Siege` : ''}.`,
    tone: 'plain',
  };
}

/** What the forecast's win chance means for this target. */
export function forecastWinLabel(f: BattleForecast, city: boolean, ranged: boolean, attackerRanged: boolean): string {
  const captures = !ranged && !attackerRanged;
  if (f.garrison && !f.garrison.shielded) return captures ? 'kill the garrison, capture' : 'destroy the garrison';
  if (city) return captures && !f.garrison ? 'capture it' : 'drop it to 0 HP';
  return 'destroy it';
}

/** Below this HP a unit next to enemies is flagged in This Turn (G18). */
const WOUNDED_HP = 50;

/**
 * Own military units below half HP that stand next to a visible military threat (barbarians are none to the Celts): flagged in
 * the This Turn dock so the player notices before an attack finishes them (G18).
 */
export function woundedNearEnemies(view: PlayerView): { unit: UnitView; text: string }[] {
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  return view.units
    .filter((u) => u.owner === view.me && u.military && u.hp < WOUNDED_HP
      && view.units.some((e) => e.military && distance(e.pos, u.pos) === 1 && isThreat(view, e.owner)))
    .sort((a, b) => a.hp - b.hp || a.id - b.id)
    .map((unit) => ({ unit, text: `${itemName({ kind: 'Unit', unit: unit.kind }, faction)} is wounded next to enemies (${unit.hp} HP)` }));
}

export const FORECAST_ASSUMPTIONS = 'Based on current queues, yields and positions. Pending orders, movement, combat and other empires’ production can change this forecast.';

export function disbandUnitLabel(view: PlayerView): string | null {
  const unit = view.economy.bankruptcy?.disband;
  if (!unit) return null;
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  const name = itemName({ kind: 'Unit', unit: unit.kind }, faction);
  if (unit.city !== null) {
    const city = view.cities.find((c) => c.id === unit.city)?.name ?? `City #${unit.city}`;
    return `${name} (completing in ${city})`;
  }
  return `${name} #${unit.unit}`;
}

export function bankruptcyWarning(view: PlayerView): string | null {
  const forecast = view.economy.bankruptcy;
  if (!forecast || forecast.gold.total >= 0) return null;
  const unit = disbandUnitLabel(view);
  return `Next turn −${Math.abs(forecast.gold.total)} Gold: ${unit ? `${unit} will be disbanded` : 'treasury will reset to 0 (no military unit to disband)'}`;
}

export function idleProductionWarning(detail: CityDetail | null): string | null {
  if (!detail || detail.producing || detail.resistance > 0 || !(detail.idle_production_waste > 0)) return null;
  return `${detail.idle_production_waste} Production will be wasted next turn (${detail.production_stored}/${detail.idle_production_cap} stored).`;
}

/**
 * Why a waiting queue's first item can't start, and what happens next (core keeps only two kinds
 * of waiting item: a unit with no free spawn tile, or an item short of its strategic resource).
 * Production meanwhile is held, up to that item's cost, and joins whatever starts next.
 */
function waitingQueueText(detail: CityDetail, cityName: string, faction?: Faction): string | null {
  const head = detail.producing ? null : detail.production_queue?.[0];
  if (!head) return null;
  const name = itemName(head, faction);
  const option = detail.options.find((o) => sameItem(o.item, head));
  const cap = detail.idle_production_cap;
  const held = `Production is held for it meanwhile (${detail.production_stored}/${cap}); a different build started now would use it instead.`;
  const waste = detail.idle_production_waste > 0 ? ` ${detail.idle_production_waste} Production over ${cap} is wasted next turn.` : '';
  if (option?.resource_blocker) return `${name} is waiting: ${option.resource_blocker.replace(/\.$/, '')}. It starts by itself once the resource is there. ${held}${waste}`;
  if (head.kind === 'Unit') return `${name} is waiting: no free tile in or beside ${cityName} for the new unit. Move a unit off a neighbouring tile and it starts once there is room. ${held}${waste}`;
  return `${name} can't start yet. ${held}${waste}`;
}

/** A short "waiting for" tag for the queue's first item ("waiting for Iron", "waiting for room"). */
export function waitingTag(detail: CityDetail): string | null {
  const head = detail.producing ? null : detail.production_queue?.[0];
  if (!head) return null;
  const blocker = detail.options.find((o) => sameItem(o.item, head))?.resource_blocker;
  const match = blocker ? /^(?:Needs \d+ (\w+)|No (\w+) to spare)/.exec(blocker) : null;
  const resource = match?.[1] ?? match?.[2];
  return resource ? `waiting for ${resource}` : head.kind === 'Unit' ? 'waiting for room to appear' : 'waiting';
}

/** A queued replacement should not inherit the old item's stall or an idle warning. */
export function cityProductionWarning(city: CityView, orders: Order[] = [], faction?: Faction, view?: PlayerView): string | null {
  const detail = city.detail;
  if (!detail) return null;
  // A frozen city has nothing to choose: its panel and card say it is occupied, never "no production chosen".
  if (city.occupation_lock != null) return null;
  const producing = effectiveProduction(orders, city.id, detail.producing);
  if (!sameItem(producing, detail.producing) && (producing || detail.producing)) return null;
  if (view && cityAutomationPreview(view, orders)?.unchosen_cities.includes(city.id)) return AUTOMATION_NO_CHOICE;
  // A lost wonder race names itself, never the next queue item (user, 2026-11-07, Thun T92).
  if (!producing && view && detail.wonder_lost && !orders.some((o) => (o.type === 'SetProductionQueue' || o.type === 'SetProduction') && o.city === city.id)) {
    return wonderLostDecision(view, city, detail, faction);
  }
  if (!producing && detail.production_queue?.length && !orders.some((o) => o.type === 'SetProductionQueue' && o.city === city.id)) {
    return waitingQueueText(detail, city.name, faction);
  }
  // A delegated city with a real choice picks at Ready, so its empty slot is not waste.
  const focus = view ? cityAutomationFocus(city.id, view, orders) : detail.build_automation;
  const stall = productionStallText(detail.production_stall);
  if (stall) return stall;
  // A met rival's wonder build and pace are public (`projects::great_work_views`): warn before the race is lost.
  if (view && producing?.kind === 'Wonder' && isGreatWork(producing.building)) {
    const own = detail.options.find((o) => sameItem(o.item, producing))?.turns ?? null;
    const race = wonderRaceWarning(view, city.name, producing.building, own);
    if (race) return race;
  }
  return focus ? null : idleProductionWarning(detail);
}

/** Bring a city that lost a wonder race, then the one wasting the most production, to the front of the Choose production CTA. */
export function idleCitiesNeedingProduction(view: PlayerView, orders: Order[]): CityView[] {
  // While Egypt works on the Royal Tomb no city builds, so none asks for a build.
  if (tombOnAfterOrders(view, orders)) return [];
  const unchosen = new Set(cityAutomationPreview(view, orders)?.unchosen_cities ?? []);
  // A city ordered razed burns at the turn boundary: it has nothing to build (8.9.29, turn 102 loop).
  const razing = new Set(orders.flatMap((o) => (o.type === 'Raze' ? [o.city] : [])));
  return view.cities.filter((c) => c.owner === view.me && c.detail && c.occupation_lock == null && !razing.has(c.id) && (unchosen.has(c.id)
    || (!cityAutomationFocus(c.id, view, orders) && !effectiveProduction(orders, c.id, c.detail.producing))))
    .sort((a, b) => Number(!!b.detail?.wonder_lost) - Number(!!a.detail?.wonder_lost)
      || (b.detail?.idle_production_waste ?? 0) - (a.detail?.idle_production_waste ?? 0) || a.id - b.id);
}

/** How close a visible enemy soldier must be for the reinforcement note: one step to reach, one to strike. */
const REINFORCE_THREAT_RANGE = 3;

/**
 * Core "reinforcing your own city goes first" (user, 2026-10-04), as a forecast note on a move into
 * your own ungarrisoned city that a visible enemy could attack this turn: your unit arrives before
 * that attack and becomes the garrison it meets. Null when the rule doesn't apply: ranged units
 * (they can't hold a city), a garrisoned city, an ally's city, or no enemy near.
 */
export function reinforcementNote(view: PlayerView, unit: Pick<UnitView, 'owner' | 'military' | 'ranged' | 'kind' | 'pos'>, target: Hex): string | null {
  if (!unit.military || !attacksInMelee(unit.kind) || unit.owner !== view.me) return null;
  const city = view.cities.find((c) => hexEq(c.pos, target) && c.owner === unit.owner);
  if (!city || hexEq(unit.pos, target) || view.units.some((u) => u.military && hexEq(u.pos, target))) return null;
  const threats = view.units.filter((u) => u.military && isThreat(view, u.owner) && distance(u.pos, target) <= REINFORCE_THREAT_RANGE);
  if (!threats.length) return null;
  const faction = view.players.find((p) => p.id === unit.owner)?.faction;
  const name = itemName({ kind: 'Unit', unit: unit.kind }, faction);
  const first = `If an enemy attacks ${city.name} this turn, your ${name} gets there first and becomes its garrison, so the attack meets it instead of an empty city.`;
  return distance(unit.pos, target) > 1
    ? `${first} It takes more than one step, so a capture on an earlier step still beats it.`
    : `${first} The city's HP absorbs the first hits.`;
}

/** `shot`: the forecast is for a shot, which never takes a city even from a unit that can charge (the Keshig). */
export function cityEntryWarning(unit: Pick<UnitView, 'kind'>, shot = false): string {
  if (shot && attacksInMelee(unit.kind)) return "A shot can't take a city. Charge it to capture it.";
  return !attacksInMelee(unit.kind) ? "Ranged units can't capture cities; move a melee unit in to take it."
    : 'Capture requires surviving and clearing enemy units. If teammates enter together, the survivor with the most HP captures; ties use a seeded coin toss. If a combined assault kills the garrison through overflow, its strongest attacker captures.';
}
