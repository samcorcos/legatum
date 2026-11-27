/**
 * Situational advice: a short reason drawn from what this seat can see, then what to do about it (user, 2026-11-25:
 * "you should think about building up your army or you might get attacked", "build a city to get iron", "purchase a
 * tile to get iron"). The failure mode is repetitive advice, so every tip is rationed: one key per situation, a
 * cooldown per key, a gap between any two tips, and a tip shows only while its situation still holds.
 */
import { distance, hexAt, hexEq } from '../hex';
import { buildingStatsFor, itemKey, itemName, sameItem, UNIT_STATS } from '../data';
import { effectiveProduction } from '../orders';
import { empireOfPlayer } from '../empires';
import { ownerName } from '../barbarians';
import { atWar, isThreat } from '../diplomacy';
import { citiesUnderAttack } from '../siege';
import type { CityView, Event, Hex, Order, PlayerView, Resource } from '../types';
import type { AdvisorStep, Suggestion } from './advisorModel';
import { OFFER_TURNS, keepPhrase, myOffer, queuedBuy, wallsInTime, withoutBuy } from '../buyCity';
import { freeGold } from '../deals';

/**
 * Turns before a tip of the same kind may show again (and only if its situation still holds). Each repeat of a kind
 * doubles its wait, so advice the player chose not to follow fades instead of nagging.
 */
export const TIP_COOLDOWN = { army: 25, war: 30, heal: 15, food: 20, resource: 20, gold: 20, buy: 20, offer: 1, garrison: 25 } as const;
/** Turns between any two different tips, so advice never piles up. (An enemy army at a city is This Turn's alarm, not a tip.) */
export const TIP_GAP = 4;
/** Turn last shown, per tip key; `#<kind>` counts how often that kind has shown this game. */
export type TipLog = Readonly<Record<string, number>>;
/** First turn this browser saw each unit locked against a rival, keyed by unit and rival id. */
export type LockLog = Readonly<Record<string, number>>;
export const tipKind = (key: string) => key.split(':')[0];
type TipKind = keyof typeof TIP_COOLDOWN;
interface Tip { key: string; kind: TipKind; title: string; why: string; steps: AdvisorStep[]; current?: number }

const ownCities = (v: PlayerView) => v.cities.filter((c) => c.owner === v.me);
const rivalTeam = (v: PlayerView, owner: number) => v.players.find((p) => p.id === owner)?.team !== v.team;
const nearest = <T extends { pos: Hex }>(items: T[], at: Hex) => [...items].sort((a, b) => distance(a.pos, at) - distance(b.pos, at))[0];
const click = (touch: boolean) => (touch ? 'Tap' : 'Click');
const tile = (hex: Hex) => ({ kind: 'tile' as const, hex });

function armyTip(v: PlayerView): Tip | null {
  if (v.turn < 10) return null;
  // Team totals: an ally's army counts as yours, and a rival's team counts as one.
  const teamArmy = (team: number) => v.players.filter((p) => p.alive && p.team === team).reduce((n, p) => n + (p.army ?? 0), 0);
  const mine = teamArmy(v.team);
  const cities = ownCities(v);
  const close = (owner: number) => v.cities.some((c) => c.owner === owner && cities.some((own) => distance(own.pos, c.pos) <= 10))
    || v.units.some((u) => u.owner === owner && u.military && cities.some((own) => distance(own.pos, u.pos) <= 6));
  const rival = v.players.filter((p) => p.alive && rivalTeam(v, p.id) && teamArmy(p.team) >= Math.max(mine * 1.5, mine + 10) && close(p.id))
    .sort((a, b) => teamArmy(b.team) - teamArmy(a.team))[0];
  if (!rival) return null;
  const name = empireOfPlayer(v, rival.id);
  // One key for every rival: the warning is about your army, not about who is ahead this time.
  return { key: 'army', kind: 'army', title: 'Build up your army',
    why: `Army points: ${name} ${teamArmy(rival.team)}, you ${mine}. They are close to you, and a weak army invites an attack.`,
    steps: [{ instruction: 'Build military units, or Walls in your border cities.', target: null }] };
}

function warTip(v: PlayerView, touch: boolean, selection: { kind: string; id: number } | null, events: readonly Event[], plan: readonly Order[] | null): Tip | null {
  if (armyTip(v)) return null;
  const cities = ownCities(v).filter((city) => city.occupation_lock == null);
  const alarmed = new Set(citiesUnderAttack(v, null).map((a) => a.cityId));
  const eligible = cities.filter((city) => !alarmed.has(city.id));
  const exposed = eligible.filter((city) =>
    !v.units.some((u) => u.military && v.players.some((p) => p.id === u.owner && p.team === v.team) && hexEq(u.pos, city.pos)));
  if (!eligible.length) return null;
  const declaration = events.find((e) => e.type === 'WarDeclared' && [e.relation.a, e.relation.b].includes(v.team)
    && !e.relation.declarers.includes(v.team) && e.relation.declarers.length > 0);
  const declaredTeam = declaration?.type === 'WarDeclared' ? declaration.relation.declarers[0] : null;
  const enemies = v.units.filter((u) => u.military && u.hp > 0 && isThreat(v, u.owner) && v.players.some((p) => p.id === u.owner && p.team !== v.team));
  const threats = exposed.flatMap((city) => enemies.filter((u) => distance(u.pos, city.pos) <= 6)
    .map((u) => ({ city, unit: u, d: distance(u.pos, city.pos) })));
  if (!declaration && !threats.length) return null;
  const closest = [...threats].sort((a, b) => a.d - b.d || a.city.id - b.city.id)[0];
  const city = closest?.city ?? eligible[0];
  const owner = closest?.unit.owner ?? v.players.find((p) => p.team === declaredTeam)?.id;
  if (owner == null) return null;
  const people = empireOfPlayer(v, owner);
  const count = enemies.filter((u) => v.players.find((p) => p.id === u.owner)?.team === v.players.find((p) => p.id === owner)?.team
    && distance(u.pos, city.pos) <= 6).length;
  const item = plan?.flatMap((o) => (o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city.id
    ? o.type === 'SetProduction' ? [o.item] : o.queue : []).find((i) => i.kind === 'Unit' && UNIT_STATS[i.unit].military);
  const build = item?.kind === 'Unit' && UNIT_STATS[item.unit].military && city.detail?.options.some((o) => itemKey(o.item) === itemKey(item) && !o.resource_blocker) ? item : null;
  const fallback = city.detail?.options.find((o) => o.item.kind === 'Unit' && UNIT_STATS[o.item.unit].military && !o.resource_blocker)?.item;
  return { key: `war:${city.id}`, kind: 'war', title: closest ? `Defend ${city.name}` : `${people} declared war`,
    why: `${people} has ${count} military ${count === 1 ? 'unit' : 'units'} near ${city.name}.`,
    current: selection?.kind === 'city' && selection.id === city.id ? 1 : 0,
    steps: [{ instruction: `${click(touch)} ${city.name}.`, target: tile(city.pos) },
      { instruction: build ? `Build ${itemName(build, v.players.find((p) => p.id === v.me)?.faction)}.` : 'Build a military unit.',
        target: build || fallback ? { kind: 'control', name: 'build', value: itemKey(build ?? fallback!) } : null }] };
}

function healTip(v: PlayerView, touch: boolean, selection: { kind: string; id: number } | null): Tip | null {
  const enemies = v.units.filter((u) => u.military && u.hp > 0 && isThreat(v, u.owner) && rivalTeam(v, u.owner));
  const unit = v.units.filter((u) => u.owner === v.me && u.military && u.hp < 40 && u.occupation_lock == null
    && enemies.some((enemy) => distance(u.pos, enemy.pos) <= 3)).sort((a, b) => a.hp - b.hp || a.id - b.id)[0];
  if (!unit) return null;
  const name = itemName({ kind: 'Unit', unit: unit.kind }, v.players.find((p) => p.id === v.me)?.faction);
  const city = nearest(ownCities(v), unit.pos);
  const canHealHere = (unit.resting_heal?.total ?? 0) > 0;
  return { key: `heal:${unit.id}`, kind: 'heal', title: `Pull your ${name} back to heal`,
    why: 'Units heal faster in your territory. A badly hurt unit can die in one attack.',
    current: selection?.kind === 'unit' && selection.id === unit.id ? 1 : 0,
    steps: [{ instruction: `${click(touch)} your ${name}.`, target: tile(unit.pos) },
      { instruction: canHealHere ? city ? `Use Heal, or move toward ${city.name}.` : 'Use Heal or move into your territory.'
        : city ? touch ? `Tap ${city.name}, then tap it again.` : `Right-click ${city.name} to move there.` : 'Move into your territory to heal.',
        target: canHealHere ? { kind: 'control', name: 'heal' } : city ? tile(city.pos) : null }] };
}

function foodTip(v: PlayerView, touch: boolean, selection: { kind: string; id: number } | null, orders: readonly Order[]): Tip | null {
  for (const city of ownCities(v)) {
    const detail = city.detail;
    if (!detail || city.occupation_lock != null) continue;
    const pending = orders.some((o) => (o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city.id);
    const producing = effectiveProduction([...orders], city.id, detail.producing);
    const changed = pending && !(sameItem(producing, detail.producing) || (!producing && !detail.producing));
    const selected = producing ? detail.options.find((o) => sameItem(o.item, producing)) : null;
    const yields = changed ? producing ? selected?.yields ?? detail.yields : detail.idle_yields ?? detail.yields : detail.yields;
    if (yields.total.food - 2 * city.population >= 0) continue;
    const faction = v.players.find((p) => p.id === v.me)?.faction;
    const option = detail.options.find((o) => o.item.kind === 'Building' && faction && (buildingStatsFor(faction, o.item.building).yields.food ?? 0) > 0
      && o.yields.total.food > yields.total.food)
      ?? detail.options.find((o) => o.item.kind === 'Project' && o.item.project === 'Food' && o.yields.total.food > yields.total.food)
      ?? detail.options.find((o) => o.item.kind === 'Improvement' && o.yields.total.food > yields.total.food);
    if (!option) continue;
    const name = itemName(option.item, v.players.find((p) => p.id === v.me)?.faction);
    return { key: `food:${city.id}`, kind: 'food', title: `${city.name} is starving`,
      why: city.population > 1 ? `${city.name} loses Food each turn and will shrink.` : `${city.name} uses more Food than it makes.`,
      current: selection?.kind === 'city' && selection.id === city.id ? 1 : 0,
      steps: [{ instruction: `${click(touch)} ${city.name}.`, target: tile(city.pos) },
        { instruction: `${option.item.kind === 'Project' ? 'Choose' : 'Build'} ${name}.`, target: { kind: 'control', name: 'build', value: itemKey(option.item) } }] };
  }
  return null;
}

function garrisonTip(v: PlayerView, locks: LockLog): Tip | null {
  const held = v.units.filter((u) => u.owner === v.me && u.occupation_lock != null
    && v.players.some((p) => p.id === u.occupation_lock && p.alive) && atWar(v, u.occupation_lock));
  const rival = held.find((u) => held.length >= 2
    || v.turn - (locks[`${u.id}:${u.occupation_lock}`] ?? v.turn) >= 10)?.occupation_lock;
  if (rival == null) return null;
  const people = ownerName(v, rival);
  return { key: `garrison:${rival}`, kind: 'garrison', title: 'Free your army',
    why: `A conquered city holds part of your army while you fight ${people}. Peace would free it.`,
    steps: [{ instruction: `Consider Peace with ${people} in Diplomacy.`, target: { kind: 'control', name: 'diplomacy' } }] };
}

function resourceTip(v: PlayerView, touch: boolean, selection: { kind: string; id: number } | null, orders: readonly Order[], placingTile: number | null): Tip | null {
  const cities = ownCities(v);
  if (!cities.length) return null;
  for (const s of v.supply ?? []) {
    if (!s.revealed) continue;
    const none = !(s.deposits?.length) && s.supplied.total <= 0;
    const short = (s.idle?.length ?? 0) > 0;
    if (!none && !short) continue;
    const r: Resource = s.resource;
    const reason = short ? `You are short of ${r}: ${s.idle![0]} needs it.` : `You have no ${r}, and some units and buildings need it.`;
    const free = v.tiles.map((t, i) => ({ t, pos: hexAt(i, v.width) })).filter(({ t }) => t.explored && t.resource === r && t.owner === null);
    // Buying a tile beside a city is the quickest way; the cheapest offer wins.
    // Only a purchase the city can make now: Gold in hand, no blocker, no tile already ordered there.
    const tileBought = orders.some((o) => o.type === 'BuyTile' && free.some((f) => hexEq(f.pos, o.tile)));
    if (tileBought) continue;
    const canBuy = (city: CityView) => !city.detail?.buy_tile_blocker && !orders.some((o) => (o.type === 'BuyTile' || o.type === 'BuyItem') && o.city === city.id);
    const offers = cities.filter(canBuy).flatMap((city) => (city.detail?.buyable_tiles ?? []).filter((b) => b.price <= v.economy.gold).filter((b) => free.some((f) => hexEq(f.pos, b.tile))).map((b) => ({ city, ...b })))
      .sort((a, b) => a.price - b.price);
    if (offers[0]) {
      const { city, price, tile: at } = offers[0];
      const inCity = selection?.kind === 'city' && selection.id === city.id;
      return { key: `resource:${r}`, kind: 'resource', title: `Buy the ${r} tile near ${city.name}`,
        why: `${reason} ${city.name} can buy it for ${price} Gold; then improve it.`, current: placingTile === city.id ? 2 : inCity ? 1 : 0,
        steps: [{ instruction: `${click(touch)} ${city.name}.`, target: tile(city.pos) },
          { instruction: `${click(touch)} Buy a tile.`, target: { kind: 'control', name: 'buyTile' } },
          { instruction: `${click(touch)} the ${r} tile.`, target: tile(at) }] };
    }
    const site = free.map((f) => ({ ...f, city: nearest(cities, f.pos) })).filter((f) => distance(f.city.pos, f.pos) <= 8)
      .sort((a, b) => distance(a.city.pos, a.pos) - distance(b.city.pos, b.pos))[0];
    if (!site) continue;
    const settler = v.units.find((u) => u.owner === v.me && u.kind === 'Settler');
    const settlerName = settler && itemName({ kind: 'Unit', unit: settler.kind }, v.players.find((p) => p.id === v.me)?.faction ?? 'Switzerland');
    return { key: `resource:${r}`, kind: 'resource', title: `Found a city near the ${r}`,
      why: `${reason} There is unclaimed ${r} near ${site.city.name}. A city beside it can claim it and improve it.`,
      steps: [settler
        ? { instruction: `Send your ${settlerName} to found a city next to the ${r}.`, target: tile(site.pos) }
        : { instruction: `Build a Settler, then found a city next to the ${r}.`, target: tile(site.pos) }] };
  }
  return null;
}

function goldTip(v: PlayerView, touch: boolean, selection: { kind: string; id: number } | null): Tip | null {
  const gold = v.economy.gold;
  // Swiss Gold earns interest in the treasury, so saving it is a real plan there.
  if (v.turn < 10 || gold < 250 || v.players.find((p) => p.id === v.me)?.faction === 'Switzerland') return null;
  // Point at your first city (usually the capital); what to buy there is the player's call.
  const city = ownCities(v)[0];
  if (!city) return null;
  return { key: 'gold', kind: 'gold', title: 'Spend your Gold',
    why: `You have ${gold} Gold. Spend it on a unit, building or tile instead of saving it.`,
    current: selection?.kind === 'city' && selection.id === city.id ? 1 : 0,
    steps: [{ instruction: `${click(touch)} ${city.name}.`, target: tile(city.pos) }, { instruction: 'Buy what it needs most.', target: null }] };
}

/** Byzantium: a rival city it can buy now with the Gold it has (core `buy_city`), cheapest first. */
function buyTip(v: PlayerView, touch: boolean, selection: { kind: string; id: number } | null, orders: readonly Order[]): Tip | null {
  if (myOffer(v) || queuedBuy(orders)) return null;
  const best = (v.city_buys ?? []).filter((b) => b.buyer === v.me && !b.blocker && b.price <= freeGold(v, withoutBuy([...orders])))
    .sort((a, b) => a.price - b.price || a.city - b.city)[0];
  const city = best && v.cities.find((c) => c.id === best.city);
  if (!best || !city) return null;
  const owner = empireOfPlayer(v, city.owner);
  const inCity = selection?.kind === 'city' && selection.id === city.id;
  return { key: 'buy', kind: 'buy', title: `You can buy ${city.name}`,
    why: `${city.name} touches your land and costs ${best.price} Gold. ${owner} gets the Gold, and the city is yours in ${OFFER_TURNS} turns, unless its owner stops it.`,
    current: inCity ? 1 : 0,
    steps: [{ instruction: `${click(touch)} ${city.name}.`, target: tile(city.pos) },
      { instruction: `${click(touch)} Make an offer.`, target: { kind: 'control', name: 'buyCity' } }] };
}

const sentence = (text: string) => text.charAt(0).toUpperCase() + text.slice(1);

/** A city of yours that Byzantium is buying: what stops it, while the offer still holds (shown every turn the offer holds, never held back by the tip gap). */
function offerTip(v: PlayerView): Tip | null {
  const o = (v.city_offers ?? []).find((x) => x.seller === v.me && x.price_now <= x.escrow);
  const city = o && v.cities.find((c) => c.id === o.city);
  if (!o || !city) return null;
  const buyer = empireOfPlayer(v, o.buyer);
  return { key: `offer:${o.city}:${o.ends_turn}`, kind: 'offer', title: `Keep ${city.name}?`,
    why: `${buyer} is buying ${city.name} for ${o.escrow} Gold, at the end of turn ${o.ends_turn}. If its price rises above ${o.escrow} Gold, the offer fails; War calls it off.`,
    steps: [{ instruction: `${sentence(keepPhrase(o.guards_to_stop ?? 0, wallsInTime(v, o)))}. To let it go, do nothing: you get the Gold.`, target: tile(city.pos) }] };
}

/** The tip to show now, or null. A tip already shown this turn keeps showing until the player dismisses it. */
export function strategyTip(view: PlayerView, saved: TipLog, touch = false, selection: { kind: string; id: number } | null = null, orders: readonly Order[] = [], placingTile: number | null = null, locks: LockLog = {}, events: readonly Event[] = [], plan: readonly Order[] | null = null): Suggestion | null {
  if (view.turn < 3) return null;
  // A log from a later turn than now belongs to another game: start clean (noteTip clears it when the next tip shows).
  const log: TipLog = Object.entries(saved).some(([k, t]) => !k.startsWith('#') && t > view.turn) ? {} : saved;
  const shownThisTurn = (key: string) => log[key] === view.turn;
  const shown = Object.entries(log).filter(([k]) => !k.startsWith('#'));
  const lastOther = (key: string) => Math.max(-Infinity, ...shown.filter(([k]) => k !== key).map(([, t]) => t));
  const lastOfKind = (kind: TipKind) => Math.max(-Infinity, ...shown.filter(([k]) => tipKind(k) === kind).map(([, t]) => t));
  const ready = (tip: Tip) => {
    if (shownThisTurn(tip.key) || tip.kind === 'offer') return true;
    const wait = TIP_COOLDOWN[tip.kind] * 2 ** Math.max(0, (log[`#${tip.kind}`] ?? 0) - 1);
    if (view.turn - lastOfKind(tip.kind) < wait) return false;
    return view.turn - lastOther(tip.key) >= TIP_GAP;
  };
  const tips = [offerTip(view), garrisonTip(view, locks), armyTip(view), warTip(view, touch, selection, events, plan), healTip(view, touch, selection), foodTip(view, touch, selection, orders), resourceTip(view, touch, selection, orders, placingTile), buyTip(view, touch, selection, orders), goldTip(view, touch, selection)].filter((t): t is Tip => !!t);
  const tip = tips.find((t) => shownThisTurn(t.key)) ?? tips.find(ready);
  if (!tip) return null;
  return { id: `tip:${tip.key}`, title: tip.title, why: tip.why, steps: tip.steps, current: tip.current ?? 0, acknowledge: true, tip: true };
}
