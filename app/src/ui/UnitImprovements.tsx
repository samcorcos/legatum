import { useId } from 'react';
import type { ReactNode } from 'react';
import type { Breakdown, Faction, PlayerView, Resource, UnitKind, UnitView } from '../types';
import { builtInSiege, MONGOL_SIEGE_REASON, itemName, SIEGE_RULE, SIEGE_USES_IRON, TECH_NAME, UNIT_STATS, unitClass, unitStatsFor } from '../data';
import { siegeTier } from '../combat';
import { aWorks, fixClause, supplyPerMine, withArticle } from '../resources';
import { strengthBonus } from '../promotion';
import { siegeSelected, upgradeSelected } from '../commands';
import { isCompany } from '../company';
import { tileIndex } from '../hex';
import { Amounts } from './Amounts';
import { goldOnHand } from './productionPurchases';
import { BreakdownTable, Icon, Kbd, Tip } from './index';
import { NotMetMark } from './bits';
import { ResourceReason } from './ResourceReason';
import { RingMedallion } from './ringMedallion';
import { Star } from '../brand/ornaments';

/** Short display copy from supply fields, only when the engine's refusal is a supply refusal.
 * Unknown refusals stay verbatim; reservation and deficit states never masquerade as "in use". */
export function improvementReason(reason: string | null, resource: Resource | null | undefined, view: PlayerView): string | null {
  if (!reason || !resource) return reason;
  const s = view.supply?.find((s) => s.resource === resource);
  if (!s || !s.revealed || s.spare > 0) return reason;
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  const noSupply = `Needs 1 ${resource}: you have none yet. ${fixClause(resource, false, faction)}`;
  const occupied = `No ${resource} to spare: all ${s.supplied.total} of your ${resource} are in use.`;
  const siegePrefix = (text: string) => `Siege: ${text[0].toLowerCase()}${text.slice(1)}`;
  if (reason !== noSupply && reason !== siegePrefix(noSupply) && !reason.startsWith(occupied) && !reason.startsWith(siegePrefix(occupied))) return reason;
  const subject = reason.startsWith('Siege: ') ? 'Siege' : 'Upgrade';
  const fix = `claim ${s.supplied.total ? `another ${resource}` : withArticle(resource)} deposit or build ${aWorks(resource)} (+${supplyPerMine(faction)})`;
  const state = !s.supplied.total ? `You have no ${resource}`
    : (s.idle?.length ?? 0) ? `You are already ${Math.max(s.used.total - s.supplied.total, s.idle?.length ?? 0)} ${resource} short`
    : (s.held?.length ?? 0) ? `You have ${s.supplied.total} ${resource}, none spare (${s.held?.length ?? 0} reserved for construction)`
    : `Your ${s.supplied.total} ${resource} are all in use`;
  return `${subject} needs 1 ${resource}. ${state}: ${fix}.`;
}

export function upgradeGain(unit: UnitView, faction: Faction, to: UpgradeCheck['to']): string {
  const next = unitStatsFor(faction, to), level = unit.level ?? 0;
  const total = (n: number) => n + strengthBonus(n, level);
  const changes = [`${total(next.strength) - total(unit.strength) >= 0 ? '+' : ''}${total(next.strength) - total(unit.strength)} Strength`];
  if (next.ranged || unit.ranged) {
    const delta = total(next.ranged) - total(unit.ranged);
    changes.push(next.ranged ? `${delta >= 0 ? '+' : ''}${delta} ranged` : 'Loses its bow');
  }
  return changes.join(' · ');
}

export function UnitImprovements({ unit, view, check, primary, pending, gold, siegeGold, siegeBlocker }: {
  unit: UnitView; view: PlayerView; check: UpgradeCheck | null;
  primary: 'upgrade' | 'siege' | null; pending?: string; gold: number; siegeGold: number; siegeBlocker?: string | null;
}) {
  const id = useId();
  const faction = view.players.find((p) => p.id === unit.owner)!.faction;
  const castrumHeal = faction === 'Rome' && unit.hp < 100
    && view.cities.some((c) => c.owner === unit.owner && c.pos.q === unit.pos.q && c.pos.r === unit.pos.r && c.detail?.buildings.includes('Walls'))
    && unitClass(unit.kind) === 'Melee' ? Math.min(50, 100 - unit.hp) : 0;
  const offer = unit.siege_offer;
  let upgradeWhy = improvementReason(check?.blocker ?? null, check?.need, view);
  // The Mongol era gate is a closed door, not a price you can't pay: it replaces the price, and the reason line steps aside.
  const siegeClosed = siegeBlocker === MONGOL_SIEGE_REASON ? `${faction} can buy Siege from the Medieval era.` : null;
  let siegeWhy = siegeClosed ? null : improvementReason(siegeBlocker ?? null, SIEGE_USES_IRON ? 'Iron' : null, view);
  // The same shortage blocks both: one line for both, not two.
  if (upgradeWhy?.startsWith('Upgrade needs ') && siegeWhy?.startsWith('Siege needs ') && upgradeWhy.slice('Upgrade'.length) === siegeWhy.slice('Siege'.length)) {
    upgradeWhy = siegeWhy = `Upgrade and Siege need ${upgradeWhy.slice('Upgrade needs '.length)}`;
  }
  const upgradeChecklist = !!check && check.needs.filter((n) => !n.met).length > 1;
  const reasons = [...new Set([upgradeChecklist ? null : upgradeWhy, siegeWhy].filter((r): r is string => !!r))];
  const upgradeReasonId = upgradeChecklist ? `${id}-upgrade-needs` : upgradeWhy ? `${id}-${reasons.indexOf(upgradeWhy)}` : undefined;
  const siegeReasonId = siegeWhy ? `${id}-${reasons.indexOf(siegeWhy)}` : undefined;
  if (!check && !offer) return null;
  return <section className="unit-improve" aria-label="Improve unit">
    <h3><Star size={10} /> Improve <span /></h3>
    {check && <ImproveRow label={`Upgrade to ${itemName({ kind: 'Unit', unit: check.to }, faction)}`}
      effect={`${upgradeGain(unit, faction, check.to)}${castrumHeal ? ` · +${castrumHeal} HP from Castrum` : ''}`} icon="ui-trend-up" hotkey="U" cost={check.cost} resource={check.need}
      primary={primary === 'upgrade'} active={pending === 'UpgradeUnit'} blocked={!!check.blocker} reasonId={upgradeReasonId} onClick={upgradeSelected}
      tip={<>
        <p>{unitStatsFor(faction, check.to).moves} moves · {unitStatsFor(faction, check.to).strength} strength before promotions{unitStatsFor(faction, check.to).ranged > 0 && ` · ${unitStatsFor(faction, check.to).ranged} ranged · range ${unitStatsFor(faction, check.to).range}`}.</p>
        <p>{castrumHeal ? `Castrum restores ${castrumHeal} HP, to ${unit.hp + castrumHeal}/100, once for this upgrade order.` : 'Keeps its HP and promotions.'} Can act again next turn.</p>
        {unit.siege && builtInSiege(check.to) && <p>Its Siege is built in from then on, so the Iron the bought Siege used is freed.</p>}
        <p>Treasury: <Amounts text={`${gold} Gold → ${gold - (check.cost ?? 0)} Gold`} />.</p>
        {unit.upgrade_breakdown && <BreakdownTable b={upgradeStepBreakdown(unit.upgrade_breakdown, unit.kind as UnitKind, unit.upgrade_steps?.length ? unit.upgrade_steps : [check.to], faction)} title="Upgrade cost" unit="Gold" plain />}
      </>} />}
    {offer && <ImproveRow label="Siege" effect={`Ignores ${['Walls', 'Walls', 'Walls and Castles', 'Walls, Castles and Star Forts'][siegeTier(view.research.known)]}`}
      icon="ui-siege" cost={offer.price.total} resource={SIEGE_USES_IRON ? 'Iron' : null}
      closed={siegeClosed} primary={primary === 'siege'} active={pending === 'UpgradeSiege'} blocked={!!siegeBlocker} reasonId={siegeClosed ? `${id}-siege-closed` : siegeReasonId} onClick={siegeSelected}
      tip={<><p>{SIEGE_RULE} The unit rests this turn.</p><p>{SIEGE_USES_IRON && 'Uses 1 Iron supply while it exists.'}</p>
        <p>Treasury: <Amounts text={`${siegeGold} Gold → ${siegeGold - offer.price.total} Gold`} />.</p>
        <BreakdownTable b={offer.price} title="Siege cost" unit="Gold" /></>} />}
    {upgradeChecklist && check && <p className="act-reason improve-reason" id={upgradeReasonId}>
      <span className="act-needs"><b>Needs</b>{check.needs.map((n) =>
        <span key={n.label} className={n.met ? 'met' : 'unmet'}>
          {check.need && n.label.includes(check.need)
            ? <ResourceReason text={n.label} resource={check.need} />
            : <Amounts text={n.label} />}{' '}
          {n.met ? <Icon name="ui-check" size={11} className="ui-mark" label="done" /> : <NotMetMark />}
        </span>)}</span>
    </p>}
    {reasons.map((reason, i) => <p className="act-reason improve-reason" id={`${id}-${i}`} key={reason}>
      {reasons.length > 1 || (upgradeChecklist && reason === siegeWhy) || (!!upgradeWhy !== !!siegeWhy && !!check && !!offer) ? !/^(Siege|Upgrade)( and Siege)? needs? /.test(reason) && <b>{reason === upgradeWhy && !upgradeChecklist ? 'Upgrade' : 'Siege'}: </b> : null}
      {((reason === upgradeWhy && check?.need && reason.includes(check.need)) || (reason === siegeWhy && SIEGE_USES_IRON && reason.includes('Iron')))
        ? <ResourceReason text={reason} resource={reason === upgradeWhy && check?.need ? check.need : 'Iron'} />
        : <Amounts text={reason} />}
    </p>)}
  </section>;
}

export function ImproveRow({ label, effect, icon, cost, resource, closed, primary, active, blocked, reasonId, onClick, tip, hotkey }: {
  label: string; effect: string; icon: string; cost: number | null; resource?: Resource | null; closed?: string | null;
  primary: boolean; active: boolean; blocked: boolean; reasonId?: string; onClick(): void; tip: ReactNode; hotkey?: string;
}) {
  return <div className={`improve-row${blocked ? ' blocked' : ''}${primary ? ' chosen-primary' : ''}`}>
    <Tip rich content={tip} summary={effect} placement="top">
      <button className={`act${primary ? ' primary' : ''}${active ? ' active' : ''}`} disabled={blocked}
        aria-describedby={reasonId} aria-pressed={active} onClick={onClick}>
        <RingMedallion icon={icon} tone="neutral" size={26} />
        <span className="improve-copy"><span>{label}</span><small>{effect}</small></span>
        {hotkey && <Kbd>{hotkey}</Kbd>}
      </button>
    </Tip>
    {closed ? <div className="improve-cost improve-closed" id={reasonId}>{closed}</div>
      : <div className="improve-cost" aria-label={`${cost === null ? 'Price unavailable' : `Costs ${cost} Gold`}${resource ? `; uses 1 ${resource}` : ''}`}>
        {cost !== null && <span><Amounts text={`−${cost} Gold`} /></span>}
        {resource && <span className="improve-supply"><Amounts text={`Uses 1 ${resource}`} /></span>}
      </div>}
  </div>;
}

export type UpgradeCheck = {
  to: UnitKind;
  /** Gold the upgrade costs now (after faction and technology modifiers). */
  cost: number | null;
  /** Every requirement, met or not, in the order the engine checks them. */
  needs: { label: string; met: boolean }[];
  /** Strategic supply the upgraded unit uses (1 of this resource), when the old kind lacks it. */
  need?: Resource | null;
  /** What to do first, when something is unmet. */
  blocker: string | null;
};

/**
 * Why an upgrade can or can't happen, checked like the engine does (the technology, the unit
 * standing on its owner's own tile, then the Gold), so the panel can name each requirement.
 * The engine reports upgrade_cost: null for every failure, so the view's own data decides.
 */
export function upgradeCheck(unit: UnitView, view: PlayerView, gold: number, reserved = new Map<Resource, number>()): UpgradeCheck | null {
  const to = unit.upgrade_to ?? UNIT_STATS[unit.kind].upgradesTo;
  if (!to) return null;
  // Core refuses it: a Company is replaced by hiring the next kind.
  if (isCompany(unit)) return { to, cost: null, needs: [], need: null, blocker: 'A hired Company cannot be upgraded: hire the next one instead' };
  const tech = UNIT_STATS[to].requires;
  const hasTech = !tech || view.research.known.includes(tech);
  const inTerritory = view.tiles[tileIndex(unit.pos, view.width, view.height)]?.owner === view.me;
  const cost = unit.upgrade_breakdown?.total ?? unit.upgrade_cost;
  const affordable = cost !== null && gold >= cost;
  // An upgrade into a unit of another era's resource also uses 1 of it while the unit exists: the core refuses them while none is spare, in its own words.
  const need = unit.upgrade_need ?? null;
  const reservedNeed = need ? reserved.get(need) ?? 0 : 0;
  const supplyBlocker = need ? unit.upgrade_supply_blocker ?? (reservedNeed >= (view.supply?.find((s) => s.resource === need)?.spare ?? 0) ? `No ${need} to spare after your queued orders.` : null) : null;
  const hasSupply = !supplyBlocker;
  const spare = need ? (view.supply?.find((s) => s.resource === need)?.spare ?? 0) - reservedNeed : 0;
  const needs = [
    ...(tech ? [{ label: TECH_NAME[tech], met: hasTech }] : []),
    { label: 'in your territory', met: inTerritory },
    ...(cost !== null ? [{ label: `${cost} Gold`, met: affordable }] : []),
    ...(need ? [{ label: `${need} supply: needs 1 spare, you have ${spare > 0 ? spare : 'none'}`, met: hasSupply }] : []),
  ];
  const blocker = !hasTech ? `Needs ${TECH_NAME[tech!]}`
    : !inTerritory ? 'Move into your territory'
    : cost !== null && !affordable ? `Needs ${cost} Gold (${goldOnHand(gold, view)})`
    : supplyBlocker;
  return { to, cost, needs, blocker, need };
}

/**
 * The engine's upgrade price as one line per step ("Spearman → Phalangite 80"), each step's faction
 * and Military Engineering parts folded into it. Every step opens with its base part (core
 * `rules::units::step_price`); a step whose base is 0 has none, so then the whole path is one line.
 */
export function upgradeStepBreakdown(b: Breakdown, from: UnitKind, steps: UnitKind[], faction?: Faction): Breakdown {
  const name = (kind: UnitKind) => itemName({ kind: 'Unit', unit: kind }, faction);
  const groups: number[] = [];
  b.parts.forEach(([label, value], i) => {
    if (label === 'Base upgrade Gold' || i === 0) groups.push(value);
    else groups[groups.length - 1] += value;
  });
  if (!steps.length || groups.length !== steps.length || b.parts[0]?.[0] !== 'Base upgrade Gold') {
    return { total: b.total, parts: [[`${name(from)} → ${name(steps.at(-1) ?? from)}`, b.total]] };
  }
  return { total: b.total, parts: steps.map((to, i) => [`${name(i ? steps[i - 1] : from)} → ${name(to)}`, groups[i]] as [string, number]) };
}
