// "Where to build this" (user, 2026-10-17): the question "which of my cities builds this fastest?" answered where the
// item is chosen, so nobody opens every city to compare. One line says the fastest city and its turns; a small list
// shows every city with its turns (progress already made counted), why one can't build it yet, and Build here / Queue
// here. The turns are the core's own (`whereToBuild.ts` picks them from the view; nothing is forecast here).
import { useMemo, type ReactNode } from 'react';
import { useGame } from '../store';
import { isGreatWork, itemKey, itemName } from '../data';
import { effectiveProduction, productionQueue, queueAfterBuild, setProductionQueue } from '../orders';
import { showsWhereToBuild, turnsWord, whereSummary, whereToBuild, type BuildSite, type WhereToBuild as Where } from '../whereToBuild';
import type { ProductionItem } from '../types';
import { feel } from '../feel';
import './whereToBuild.css';

/** The sites of one item for the viewer, from the live view and the orders given this turn. */
function useWhereToBuild(item: ProductionItem): Where | null {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const orders = useGame((s) => s.orders[s.seat]);
  const key = itemKey(item);
  return useMemo(() => (view ? whereToBuild(view, item, orders ?? []) : null), [view, orders, key]);
}

function siteNote(s: BuildSite, cost: number): ReactNode {
  if (s.state === 'blocked' || s.state === 'elsewhere') return <span className="wtb-reason">{s.reason}</span>;
  const parts: ReactNode[] = [];
  parts.push(s.turns === null ? <span key="t" className="wtb-turns none">No output</span> : <b key="t" className="wtb-turns">{turnsWord(s.turns)}</b>);
  if (s.state === 'later') parts.push(<span key="s" className="wtb-sub">once researched</span>);
  else if (s.state === 'building') parts.push(<span key="s" className="wtb-sub">{s.pending ? 'chosen, starts when the turn ends' : 'building it now'}</span>);
  else if (s.state === 'queued') parts.push(<span key="s" className="wtb-sub">queued #{s.position}</span>);
  else if (s.stored > 0) parts.push(<span key="s" className="wtb-sub">{s.stored}/{cost} already made</span>);
  if (s.replaces) parts.push(<span key="r" className="wtb-sub" title={`${s.replaces} stops for now. Its progress stays banked in this city and resumes when you build it again.`}>pauses {s.replaces}, progress kept</span>);
  else if (s.idle && s.state === 'ready') parts.push(<span key="i" className="wtb-sub">city is idle</span>);
  return <>{parts.map((p, i) => <span key={i} className="wtb-bit">{p}</span>)}</>;
}

/**
 * The line and the list. `here` is the city whose panel it sits in; `preview` is the read-only version for a
 * technology not yet researched (no buttons); `hideNone` drops the line when no city can build it; `onEdit` lets the panel keep the pointer's row in place when the
 * queue above it grows.
 */
export function WhereToBuild({ item, here, preview = false, onEdit, open, hideNone = false }: { item: ProductionItem; here?: number; preview?: boolean; onEdit?: (from: Element) => void; open?: boolean; hideNone?: boolean }) {
  const w = useWhereToBuild(item);
  const editOrders = useGame((s) => s.editOrders);
  const finished = useGame((s) => !!s.views?.[s.seat]?.finished);
  const faction = useGame((s) => s.views?.[s.seat]?.players.find((p) => p.id === s.views![s.seat].me)?.faction);
  if (!w || !showsWhereToBuild(w)) return null;
  const summary = whereSummary(w, here);
  // A greyed card already says why; "no city can build it" under it adds nothing.
  if (hideNone && summary.kind === 'none') return null;
  const name = itemName(item, faction);
  const needsTile = (item.kind === 'Building' || item.kind === 'Wonder') && (isGreatWork(item.building) || item.building === 'CourierHall');
  const edit = (site: BuildSite, mode: 'build' | 'queue', from: Element) => {
    onEdit?.(from);
    feel.select();
    editOrders((orders, view) => {
      const city = view.cities.find((c) => c.id === site.cityId);
      const d = city?.detail;
      if (!d) return orders;
      const queue = productionQueue(orders, site.cityId, d.producing, d.production_queue);
      const next = mode === 'build' ? queueAfterBuild(queue, effectiveProduction(orders, site.cityId, d.producing), item) : [...queue, item];
      return setProductionQueue(orders, site.cityId, next);
    }, 'queue');
  };
  const goTo = (site: BuildSite) => useGame.getState().select({ kind: 'city', id: site.cityId }, true);
  return (
    <details className={`wtb ${summary.kind}`} data-testid="where-to-build" data-item={itemKey(item)} open={open}>
      <summary>
        {/* Once the list is open and scrolled, its sticky head still says which item it is about. */}
        <span className="wtb-item">{name}</span>
        <span className="wtb-line" data-testid="where-line">{summary.text}</span>
        <span className="wtb-toggle">Compare all {w.sites.length} cities</span>
      </summary>
      <ul className="wtb-list" aria-label={`Where to build ${name}`}>
        {w.sites.map((s) => {
          const canAct = !preview && w.researched && !finished && (s.state === 'ready' || s.state === 'queued');
          return (
            <li key={s.cityId} className={`wtb-row ${s.state}`} data-testid="wtb-row" data-city={s.cityId} data-state={s.state}>
              <span className="wtb-who">
                <button type="button" className="wtb-city" onClick={() => goTo(s)} title={`Go to ${s.name}`}>{s.name}</button>
                
              </span>
              <span className="wtb-what">{w.fastest && s.turns === w.fastest.turns && ['ready', 'building', 'queued', 'later'].includes(s.state) && <span className="wtb-tag">Fastest</span>}{siteNote(s, w.cost)}</span>
              {canAct && <span className="wtb-actions">
                {needsTile ? (
                  <button type="button" className="wtb-act" aria-label={`Choose tile in ${s.name}: ${name}`} onClick={() => goTo(s)}>Choose tile in {s.name}</button>
                ) : <>
                  <button type="button" className="wtb-act" aria-label={`Build here in ${s.name}: ${name}`} onClick={(e) => edit(s, 'build', e.currentTarget)}>Build here</button>
                  {s.state === 'ready' && <button type="button" className="wtb-act" aria-label={`Queue here in ${s.name}: ${name}`} onClick={(e) => edit(s, 'queue', e.currentTarget)}>Queue here</button>}
                </>}
              </span>}
            </li>
          );
        })}
      </ul>
    </details>
  );
}

/** The read-only version for a hovercard: the line and the first few cities as plain text (a hovercard is never clicked). */
export function WhereToBuildLine({ item }: { item: ProductionItem }) {
  const w = useWhereToBuild(item);
  if (!w || !showsWhereToBuild(w)) return null;
  const summary = whereSummary(w);
  const shown = w.sites.slice(0, 4);
  return (
    <div className="uc-where" data-testid="uc-where">
      <div className="uc-vs">Where to build it</div>
      <p><b>{summary.text}</b></p>
      <ul>
        {shown.map((s) => <li key={s.cityId}>{s.name}: {s.turns !== null ? turnsWord(s.turns) : s.reason ?? 'No output'}</li>)}
        {w.sites.length > shown.length && <li>and {w.sites.length - shown.length} more</li>}
      </ul>
    </div>
  );
}
