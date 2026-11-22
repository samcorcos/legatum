// Every revealed strategic supply has its own top-bar chip and breakdown.
// The tooltip is the whole sum (pillar 5): which deposits supply it, what needs it, what builds in
// progress hold, what is idle (each with Show on map) and what is spare or short. A shortage shows an alert mark and the idle count,
// never colour alone. The numbers and every line come from the core's `SupplyView`.
import { RESOURCE_INFO, STRATEGIC_RESOURCES, UNSUPPLIED_STRENGTH_PERCENT, aWorks, depositHolders, empireHolds, eraResource, fixClause, idleCountText, idleEntries, resourceUsersThisEra, resourceUsers, revealHint, supplyAtWork, supplyRuleText, type IdleEntry } from '../resources';
import type { Breakdown, Hex, PlayerView, Resource, SupplyView } from '../types';
import { placeName } from '../empires';
import { BreakdownTable, Tip } from './index';
import { Icon, MaskIcon } from './bits';
import { iconUrl, resourceArtUrl } from '../icons';
import { Amounts } from './Amounts';
import { useGame } from '../store';
import { TECH_NAME, UNIT_STATS } from '../data';

/** True when the buildings and units that need it outnumber the supply: the newest are idle. */
const isShort = (s: SupplyView): boolean => (s.idle?.length ?? 0) > 0;

/**
 * The supply is revealed and none of it is inside your borders, while something here waits on it: a building or unit
 * already short (idle), a build in progress holding it, or a unit whose upgrade needs it.
 */
function blockedUpgrade(view: Pick<PlayerView, 'units' | 'me' | 'research'>, resource: Resource): boolean {
  return view.units.some((u) => {
    if (u.owner !== view.me || u.upgrade_need !== resource || !u.upgrade_supply_blocker) return false;
    const to = UNIT_STATS[u.kind].upgradesTo;
    const tech = to && UNIT_STATS[to].requires;
    return !!to && (!tech || view.research.known.includes(tech));
  });
}

export function isMissing(view: Pick<PlayerView, 'units' | 'me' | 'research'>, s: SupplyView): boolean {
  if (!s.revealed || s.supplied.total > 0) return false;
  return s.used.total > 0 || (s.held?.length ?? 0) > 0 || blockedUpgrade(view, s.resource);
}

/** "Missing Niter: your gunpowder units and Banks need it." */
export const missingText = (r: Resource, faction?: string): string => `Missing ${RESOURCE_INFO[r].name}: ${resourceUsers(r, faction) ?? 'your buildings and units'} need it.`;

/** Lead with the unclaimed deposits, then name the empires that hold others. */
export function holdersText(view: Pick<PlayerView, 'tiles' | 'players' | 'me' | 'width' | 'cities' | 'units'>, r: Resource): string {
  const name = RESOURCE_INFO[r].name;
  const { empires, held, unclaimed, nearestUnclaimed } = depositHolders(view, r);
  const deposits = (n: number) => `${n} ${name} deposit${n === 1 ? '' : 's'}`;
  const place = nearestUnclaimed && placeName(view, nearestUnclaimed);
  const parts: string[] = [];
  if (unclaimed) {
    const where = place ? unclaimed === 1 ? ` ${place}` : `; the nearest is ${place}` : '';
    parts.push(unclaimed === 1 ? `One unclaimed ${name} deposit lies${where}.` : `${unclaimed} unclaimed ${name} deposits lie${where}.`);
  }
  if (held) parts.push(unclaimed
    ? `${empireHolds(empires)} ${held === 1 ? `another ${name} deposit` : `${held} other ${name} deposits`}.`
    : `${empireHolds(empires)} ${deposits(held)} you have seen.`);
  return parts.length ? parts.join(' ') : `You have seen no ${name} deposit yet. Explore for one.`;
}

/** "1/6": free supply over total supply. Negative spare is shown as zero; the idle count explains the shortage. */
export const supplyFraction = (s: SupplyView): string => `${Math.max(0, s.spare)}/${s.supplied.total}`;

/** The one-line hover text: the sum, and what is idle when short. */
export function supplySummary(s: SupplyView): string {
  const name = RESOURCE_INFO[s.resource].name;
  if (!s.revealed) return revealHint(s.resource);
  const idle = s.idle?.length ?? 0;
  if (idle > 0) return `${name}: needs ${s.used.total}, supplies ${s.supplied.total}, so ${idleCountText(idle)}`;
  if (s.supplied.total === 0) return `${name}: no deposit inside your borders yet`;
  const spare = Math.max(0, s.spare);
  return `${name}: ${supplyAtWork(s)} of ${s.supplied.total} at work, ${spare} to spare`;
}

/** Each deposit by its tile, so two near one city read apart: "Coal with a Mine, 3 tiles N of Aarau" ("Oil with an Oil Well"). Germany's deposits supply double: the usual number, then the doubling as its own signed row. */
function depositRows(s: SupplyView, view?: Pick<PlayerView, 'cities'>, faction?: string): Breakdown {
  const name = RESOURCE_INFO[s.resource].name;
  if (!view || !s.deposits?.length) return s.supplied;
  const parts: [string, number][] = [];
  for (const d of s.deposits) {
    const label = `${name}${d.mined ? ` with ${aWorks(s.resource)}` : ''}, ${placeName(view, d.hex) ?? 'in your land'}`;
    if (faction === 'Germany' && d.supply % 2 === 0) parts.push([label, d.supply / 2], [GERMANY_DOUBLE_ROW, d.supply / 2]);
    else parts.push([label, d.supply]);
  }
  return { total: s.supplied.total, parts };
}
const GERMANY_DOUBLE_ROW = 'Germany: deposits supply double';

/** The missing-resource alert: what needs it, who holds the deposits you have seen, and the nearest one to look at. */
export interface MissingNote { users: string; holders: string; target: Hex | null }

export function SupplyTip({ s, idleEntries: entries, view, missing, era }: { s: SupplyView; idleEntries?: IdleEntry[]; view?: Pick<PlayerView, 'cities' | 'players' | 'me'>; missing?: MissingNote; era?: number }) {
  const faction = view?.players.find((p) => p.id === view.me)?.faction;
  const name = RESOURCE_INFO[s.resource].name;
  if (!s.revealed) return <div className="bd supply-tip supply-reveal-tip"><div className="bd-title">{revealHint(s.resource)}</div><div className="bd-note supply-users">Used for this era: {resourceUsersThisEra(s.resource, era ?? 0, faction)}.</div></div>;
  const idle = s.idle ?? [];
  const held = s.held ?? [];
  const list = entries ?? idle.map((label) => ({ label, hex: null }));
  return <div className="bd supply-tip">
    {missing
      ? <>
        <div className="bd-title supply-missing-title"><Icon name="ui-alert" size={12} className="ui-mark" /> Missing {name}</div>
        <div className="bd-note supply-short-note supply-missing-note">{missing.users} <span className="supply-holders">{missing.holders}</span>
          {missing.target && <> <button type="button" className="link supply-show-map" onClick={() => useGame.getState().jumpTo(missing.target!, true)}>Show on map</button></>}
        </div>
      </>
      : <div className="bd-title"><Amounts text={`${supplyAtWork(s)} of ${s.supplied.total} ${name}`} /> at work{idle.length > 0 && `, ${idle.length} idle`}</div>}
    <div className="bd-note supply-users">Used for{era !== undefined ? ' this era' : ''}: {era === undefined ? resourceUsers(s.resource, faction) ?? 'buildings and units' : resourceUsersThisEra(s.resource, era, faction)}.</div>
    {idle.length > 0 && <div className="bd-note supply-short-note">
      <Icon name="ui-alert" size={12} className="ui-mark" /> Short of {name}: {idleCountText(idle.length)}. An idle building yields nothing and an idle unit fights {UNSUPPLIED_STRENGTH_PERCENT}% weaker until supply returns. The newest go idle first.
    </div>}
    {s.supplied.parts.length > 0
      ? <BreakdownTable b={depositRows(s, view, faction)} title={`${name} supplied`} plain signedLabels={[GERMANY_DOUBLE_ROW]} />
      : <div className="bd-note">No {name} deposit inside your borders yet. <Amounts text={fixClause(s.resource, false, faction)} /></div>}
    {s.used.parts.length > 0 && <BreakdownTable b={s.used} title={`${name} needed`} plain />}
    {held.length > 0 && <div className="bd-row supply-held"><span>{held.map((label, i) => <span key={`${label}-${i}`}>{i > 0 ? ', ' : ''}1 held for the {label}</span>)}</span><span>{held.length}</span></div>}
    {list.length > 0 && <div className="bd-note neg supply-idle-list">
      <div>Idle, newest first:</div>
      <ul>{list.map((e, i) => <li key={`${e.label}-${i}`}>
        <span>{e.label}</span>
        {e.hex && <button type="button" className="link supply-show-map" onClick={() => useGame.getState().jumpTo(e.hex!, true)}>Show on map</button>}
      </li>)}</ul>
    </div>}
    {idle.length > 0 && <div className="bd-row neg supply-totals"><span>Needs {s.used.total}, supplies {s.supplied.total}</span><b>Short by {idle.length}</b></div>}
    <div className="bd-row bd-total"><span>Spare</span><span>{Math.max(0, s.spare)}</span></div>
    <div className="bd-note"><Amounts text={`${supplyRuleText(s.resource, faction)} Each building or unit that needs ${name} uses 1 while it exists.`} /> Supply is counted at the end of every turn.</div>
  </div>;
}

/** The shared painted resource art at both chip sizes; 32 px sources avoid needless downscaling on small chips. */
export function SupplyIcon({ resource, size }: { resource: string; size: number }) {
  const name = `resource-${resource.toLowerCase()}`;
  const art = resourceArtUrl(resource, size < 20 ? 32 : 64);
  if (art) return <img className="supply-resource-icon supply-art" src={art} width={size} height={size} alt={resource} draggable={false} />;
  const url = iconUrl(name);
  return url
    ? <MaskIcon url={url} size={size} className="supply-resource-icon" color="var(--supply-ink, #b4533f)" label={resource} />
    : <Icon name={name} size={size} fallback={<span className="supply-glyph" aria-hidden>{resource[0]}</span>} />;
}

const viewerFaction = (view: Pick<PlayerView, 'players' | 'me'>): string | undefined => view.players.find((p) => p.id === view.me)?.faction;

/** The missing-resource alert for one resource, as the tooltip and the hover line say it. */
function missingNote(view: PlayerView, r: Resource): MissingNote {
  return { users: `${(resourceUsers(r, viewerFaction(view)) ?? 'your buildings and units').replace(/^./, (c) => c.toUpperCase())} need it.`, holders: holdersText(view, r), target: depositHolders(view, r).nearest };
}

/** The same in one line: "Missing Niter: your gunpowder units and Banks need it. Switzerland holds 1 Niter deposit you have seen." */
const missingLine = (view: PlayerView, r: Resource): string => `${missingText(r, viewerFaction(view))} ${holdersText(view, r)}`;

/** The viewer's era resource: from the research view's 0-based era, the same era the era chip shows. */
const eraResourceOf = (view: Pick<PlayerView, 'research'>): Resource | null => eraResource(view.research.era);

/** Every revealed strategic resource, plus the current era's hidden resource as a research target. */
export function chipSupplies(view: PlayerView): SupplyView[] {
  const mine = eraResourceOf(view);
  return (view.supply ?? []).filter((s) => (s.revealed || s.resource === mine) && STRATEGIC_RESOURCES.includes(s.resource) && view.cities.some((c) => c.owner === view.me))
    .sort((a, b) => STRATEGIC_RESOURCES.indexOf(a.resource) - STRATEGIC_RESOURCES.indexOf(b.resource));
}

/** A resource is wanted if an existing user, a held build, or an unlocked upgrade is blocked by it. */
export function supplyWanted(view: PlayerView, s: SupplyView): boolean {
  return s.used.total > 0 || !!s.held?.length
    || blockedUpgrade(view, s.resource)
    || view.cities.some((c) => c.owner === view.me && c.detail?.options.some((o) => o.supply_need === s.resource && !!o.resource_blocker));
}

/** Chips for revealed supplies and the current era's unrevealed research target. */
export function SupplyChips({ view }: { view: PlayerView }) {
  const chips = chipSupplies(view);
  if (!chips.length) return null;
  const mine = eraResourceOf(view);
  return <div className="supplies">
    {chips.map((s) => {
      const name = RESOURCE_INFO[s.resource].name;
      if (!s.revealed) return <Tip key={s.resource} content={<SupplyTip s={s} view={view} era={view.research.era} />}
        className={`stat supply-stat supply-${s.resource.toLowerCase()} supply-era supply-unrevealed`}>
        <SupplyIcon resource={s.resource} size={24} />
        <span className="supply-reveal-label"><span>{name}: revealed by</span><span> {RESOURCE_INFO[s.resource].revealedBy ? TECH_NAME[RESOURCE_INFO[s.resource].revealedBy!] : 'research'}</span></span>
      </Tip>;
      const blockedBuild = view.cities.some((c) => c.owner === view.me && c.detail?.options.some((o) => o.supply_need === s.resource && !!o.resource_blocker));
      const upgradeShort = blockedUpgrade(view, s.resource);
      const short = s.used.total > s.supplied.total || isShort(s) || (s.spare <= 0 && (blockedBuild || upgradeShort));
      const missing = isMissing(view, s);
      return <Tip key={s.resource} rich content={<SupplyTip s={s} idleEntries={idleEntries(view, s)} view={view} era={view.research.era} missing={missing ? missingNote(view, s.resource) : undefined} />}
        summary={`${name}: ${supplyFraction(s)} spare/total. ${missing ? missingLine(view, s.resource) : supplySummary(s)}`}
        className={`stat supply-stat supply-${s.resource.toLowerCase()}${s.resource === mine ? ' supply-era' : ''}${short || missing ? ' supply-short' : ''}${missing ? ' supply-missing' : ''}`}>
        <SupplyIcon resource={s.resource} size={24} />
        <span className="supply-name">{name}</span>
        {(short || missing) && <Icon name="ui-alert" size={13} className="ui-mark supply-alert" label={missing ? `Missing ${name}` : `No ${name} to spare`} />}
        <b data-testid={`supply-${s.resource}`} data-spare={Math.max(0, s.spare)} aria-label={`${name}: ${supplyFraction(s)} spare and total`}>{supplyFraction(s)}</b>
        {isShort(s) && <span className="supply-idle neg">{s.idle!.length} idle</span>}
      </Tip>;
    })}
  </div>;
}
