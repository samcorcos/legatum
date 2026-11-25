/**
 * Situational advice: a short reason drawn from what this seat can see, then what to do about it (user, 2026-11-25:
 * "you should think about building up your army or you might get attacked", "build a city to get iron", "purchase a
 * tile to get iron"). The failure mode is repetitive advice, so every tip is rationed: one key per situation, a
 * cooldown per key, a gap between any two tips, and a tip shows only while its situation still holds.
 */
import { isThreat } from '../diplomacy';
import { distance, hexAt, hexEq } from '../hex';
import { itemName } from '../data';
import { empireOfPlayer } from '../empires';
import type { CityView, Hex, Order, PlayerView, Resource } from '../types';
import type { AdvisorStep, Suggestion } from './advisorModel';

/**
 * Turns before a tip of the same kind may show again (and only if its situation still holds). Each repeat of a kind
 * doubles its wait, so advice the player chose not to follow fades instead of nagging.
 */
export const TIP_COOLDOWN = { threat: 10, army: 25, resource: 20, gold: 20 } as const;
/** Turns between any two different tips, so advice never piles up. An enemy at the gates skips the wait. */
export const TIP_GAP = 4;
/** Turn last shown, per tip key; `#<kind>` counts how often that kind has shown this game. */
export type TipLog = Readonly<Record<string, number>>;
export const tipKind = (key: string) => key.split(':')[0];
type TipKind = keyof typeof TIP_COOLDOWN;
interface Tip { key: string; kind: TipKind; title: string; why: string; steps: AdvisorStep[]; current?: number }

const ownCities = (v: PlayerView) => v.cities.filter((c) => c.owner === v.me);
const rivalTeam = (v: PlayerView, owner: number) => v.players.find((p) => p.id === owner)?.team !== v.team;
const nearest = <T extends { pos: Hex }>(items: T[], at: Hex) => [...items].sort((a, b) => distance(a.pos, at) - distance(b.pos, at))[0];
const possessive = (name: string) => (name.endsWith('s') ? `${name}'` : `${name}'s`);
const click = (touch: boolean) => (touch ? 'Tap' : 'Click');
const tile = (hex: Hex) => ({ kind: 'tile' as const, hex });

function threatTip(v: PlayerView, touch: boolean, selection: { kind: string; id: number } | null): Tip | null {
  let worst: { city: CityView; enemies: number; guards: number } | null = null;
  for (const city of ownCities(v)) {
    const enemies = v.units.filter((u) => u.military && isThreat(v, u.owner) && distance(u.pos, city.pos) <= 3).length;
    const guards = v.units.filter((u) => u.owner === v.me && u.military && distance(u.pos, city.pos) <= 1).length;
    if (enemies >= 2 && enemies > guards && (!worst || enemies - guards > worst.enemies - worst.guards)) worst = { city, enemies, guards };
  }
  if (!worst) return null;
  const { city, enemies, guards } = worst;
  return { key: `threat:${city.id}`, kind: 'threat', title: `Defend ${city.name}`,
    why: `${enemies} enemy ${enemies === 1 ? 'unit is' : 'units are'} within 3 tiles of ${city.name}, and ${guards === 0 ? 'none of yours guard it' : `${guards} of yours ${guards === 1 ? 'guards' : 'guard'} it`}. Build or buy a military unit there.`,
    current: selection?.kind === 'city' && selection.id === city.id ? 1 : 0,
    steps: [{ instruction: `${click(touch)} ${city.name}.`, target: tile(city.pos) }, { instruction: 'Build or buy a military unit.', target: null }] };
}

function armyTip(v: PlayerView): Tip | null {
  if (v.turn < 10) return null;
  const mine = v.players.find((p) => p.id === v.me)?.army ?? 0;
  const cities = ownCities(v);
  const close = (owner: number) => v.cities.some((c) => c.owner === owner && cities.some((own) => distance(own.pos, c.pos) <= 10))
    || v.units.some((u) => u.owner === owner && u.military && cities.some((own) => distance(own.pos, u.pos) <= 6));
  const rival = v.players.filter((p) => p.alive && rivalTeam(v, p.id) && (p.army ?? 0) >= Math.max(mine * 1.5, mine + 10) && close(p.id))
    .sort((a, b) => (b.army ?? 0) - (a.army ?? 0))[0];
  if (!rival) return null;
  const name = empireOfPlayer(v, rival.id);
  // One key for every rival: the warning is about your army, not about who is ahead this time.
  return { key: 'army', kind: 'army', title: 'Build up your army',
    why: `${possessive(name)} army strength is ${rival.army}; yours is ${mine}. They are close to you, and a weak army invites an attack.`,
    steps: [{ instruction: 'Build military units, or Walls in your border cities.', target: null }] };
}

function resourceTip(v: PlayerView, touch: boolean, selection: { kind: string; id: number } | null, orders: readonly Order[]): Tip | null {
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
    const canBuy = (city: CityView) => !city.detail?.buy_tile_blocker && !orders.some((o) => (o.type === 'BuyTile' || o.type === 'BuyItem' || o.type === 'HireCompany') && o.city === city.id);
    const offers = cities.filter(canBuy).flatMap((city) => (city.detail?.buyable_tiles ?? []).filter((b) => b.price <= v.economy.gold).filter((b) => free.some((f) => hexEq(f.pos, b.tile))).map((b) => ({ city, ...b })))
      .sort((a, b) => a.price - b.price);
    if (offers[0]) {
      const { city, price } = offers[0];
      const inCity = selection?.kind === 'city' && selection.id === city.id;
      return { key: `resource:${r}`, kind: 'resource', title: `Buy the ${r} tile near ${city.name}`,
        why: `${reason} ${city.name} can buy it for ${price} Gold; then improve it.`, current: inCity ? 1 : 0,
        steps: [{ instruction: `${click(touch)} ${city.name}.`, target: tile(city.pos) },
          { instruction: `${click(touch)} Buy a tile, then the ${r} tile.`, target: { kind: 'control', name: 'buyTile' } }] };
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

function goldTip(v: PlayerView): Tip | null {
  const gold = v.economy.gold;
  // Swiss Gold earns interest in the treasury, so saving it is a real plan there.
  if (v.turn < 10 || gold < 250 || v.players.find((p) => p.id === v.me)?.faction === 'Switzerland') return null;
  return { key: 'gold', kind: 'gold', title: 'Spend your Gold',
    why: `You have ${gold} Gold. Buying a unit, building or tile turns it into strength now.`,
    steps: [{ instruction: 'Open a city and use Buy on what it needs most.', target: null }] };
}

/** The tip to show now, or null. A tip already shown this turn keeps showing until the player dismisses it. */
export function strategyTip(view: PlayerView, saved: TipLog, touch = false, selection: { kind: string; id: number } | null = null, orders: readonly Order[] = []): Suggestion | null {
  if (view.turn < 3) return null;
  // A log from a later turn than now belongs to another game: start clean (noteTip clears it when the next tip shows).
  const log: TipLog = Object.entries(saved).some(([k, t]) => !k.startsWith('#') && t > view.turn) ? {} : saved;
  const shownThisTurn = (key: string) => log[key] === view.turn;
  const shown = Object.entries(log).filter(([k]) => !k.startsWith('#'));
  const lastOther = (key: string) => Math.max(-Infinity, ...shown.filter(([k]) => k !== key).map(([, t]) => t));
  const lastOfKind = (kind: TipKind) => Math.max(-Infinity, ...shown.filter(([k]) => tipKind(k) === kind).map(([, t]) => t));
  const ready = (tip: Tip) => {
    if (shownThisTurn(tip.key)) return true;
    const wait = TIP_COOLDOWN[tip.kind] * 2 ** Math.max(0, (log[`#${tip.kind}`] ?? 0) - 1);
    if (view.turn - lastOfKind(tip.kind) < wait) return false;
    return tip.kind === 'threat' || view.turn - lastOther(tip.key) >= TIP_GAP;
  };
  const tips = [threatTip(view, touch, selection), armyTip(view), resourceTip(view, touch, selection, orders), goldTip(view)].filter((t): t is Tip => !!t);
  const tip = tips.find((t) => shownThisTurn(t.key)) ?? tips.find(ready);
  if (!tip) return null;
  return { id: `tip:${tip.key}`, title: tip.title, why: tip.why, steps: tip.steps, current: tip.current ?? 0, acknowledge: true, tip: true };
}
