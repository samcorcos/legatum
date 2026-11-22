// Mercenary Companies (Byzantium), as the UI reads them: the shop (`PlayerView.company_offers`), a hired unit's contract
// (`UnitView.contract_until`, `origin`) and an empire's roster (`PlayerPublic.companies`). Every number comes from the core
// view or `RULES.company`; nothing is recomputed here except words.
import { RULES, factionName, itemName, unitStatsFor } from './data';
import type { CityId, CompanyOffer, Faction, Order, PlayerId, PlayerView, UnitKind, UnitView } from './types';

const turnsWord = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;

/** Turns of contract left on a hired unit: 0 on its last turn. Null for an ordinary unit. */
export function contractLeft(unit: Pick<UnitView, 'contract_until'>, view: Pick<PlayerView, 'turn'>): number | null {
  return unit.contract_until === undefined || unit.contract_until === null ? null : Math.max(0, unit.contract_until - view.turn);
}

export const isCompany = (unit: Pick<UnitView, 'contract_until'>) => unit.contract_until !== undefined && unit.contract_until !== null;

/** "Sparta" for a Company sold by Sparta ("another empire" when the seller is unknown). */
const originName = (unit: Pick<UnitView, 'origin'>) => (unit.origin ? factionName(unit.origin) : 'another empire');

/** The short mark under a unit on the map: "7 left", or "Last turn". */
export const contractTag = (left: number) => (left === 0 ? 'Last turn' : `${left} turn${left === 1 ? '' : 's'}`);

/** The unit panel's one line: "Hired from Sparta: 7 turns of contract left, upkeep 2 Gold a turn". */
export function contractLine(unit: UnitView, view: Pick<PlayerView, 'turn'>): string {
  const left = contractLeft(unit, view);
  if (left === null) return '';
  const head = `Hired from ${originName(unit)}`;
  return left === 0
    ? `${head}: goes home this turn`
    : `${head}: ${turnsWord(left)} of contract left, upkeep ${RULES.company.upkeep} Gold a turn`;
}

/** The sentence that explains a Company to someone who has not read the Guide. */
export const COMPANY_UNIT_RULE = `A Company serves ${RULES.company.turns} turns, then goes home. It fights and captures like any unit, costs ${RULES.company.upkeep} Gold a turn, cannot be upgraded and does not count against thin levies. If its owner cannot pay the upkeep, it turns barbarian where it stands.`;

/** An empire's Companies the viewer can see, soonest contract first. */
function companiesOf(view: PlayerView, owner: PlayerId): UnitView[] {
  return view.units.filter((u) => u.owner === owner && isCompany(u))
    .sort((a, b) => (a.contract_until ?? 0) - (b.contract_until ?? 0) || a.id - b.id);
}

/** "Companies: 3 · upkeep 6 Gold a turn · next goes home in 4 turns" for an empire (the roster count is public; the countdown needs its units in sight). */
export function rosterLine(view: PlayerView, owner: PlayerId, count: number): string {
  const seen = companiesOf(view, owner);
  const upkeep = `upkeep ${count * RULES.company.upkeep} Gold a turn`;
  const next = seen.length === count ? contractLeft(seen[0], view) : null;
  const home = next === null ? '' : next === 0 ? ' · next goes home this turn' : ` · next goes home in ${turnsWord(next)}`;
  return `Companies: ${count} · ${upkeep}${home}`;
}

/** What a hired unit is: strength, shot and moves in its seller's own version (core `company`: the Company is the seller's unit). */
export function companyStats(seller: Faction, kind: UnitKind): string {
  const s = unitStatsFor(seller, kind);
  return `${s.strength} melee${s.ranged ? `, ${s.ranged} ranged at range ${s.range}` : ''}, ${s.moves} moves`;
}

/** The seller's name for the unit ("Eagle Warrior" from the Aztecs). */
export const companyName = (seller: Faction, kind: UnitKind) => itemName({ kind: 'Unit', unit: kind }, seller);

/** The shop, one row per seller in id order, with the seller's faction (an offer whose seller is unknown is left out). */
export interface ShopRow { offer: CompanyOffer; faction: Faction }
export function shopRows(view: PlayerView): ShopRow[] {
  return [...(view.company_offers ?? [])].sort((a, b) => a.seller - b.seller).flatMap((offer) => {
    const faction = view.players.find((p) => p.id === offer.seller)?.faction;
    return faction ? [{ offer, faction }] : [];
  });
}

/** Whether the viewer is a faction that hires at all (the shop section exists only for it). */
export const canHire = (view: PlayerView) => view.players.find((p) => p.id === view.me)?.faction === 'Byzantium';

/** The hire order for one row. */
export type HireOrder = Extract<Order, { type: 'HireCompany' }>;
export const hireOrder = (city: CityId, offer: CompanyOffer): HireOrder => ({ type: 'HireCompany', city, unit: offer.unit, seller: offer.seller });

/** This city's pending hire, if any. */
export const pendingHire = (orders: Order[], city: CityId) => orders.find((o): o is HireOrder => o.type === 'HireCompany' && o.city === city);
