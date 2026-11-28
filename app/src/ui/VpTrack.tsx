import { useMemo, useRef } from 'react';
import { relationWith } from '../diplomacy';
import { nearestRival, rivalLine } from '../rivalWin';
import { RivalWin } from './RivalWin';
import { pendingDecisions } from '../turnFlow';
import { useGame } from '../store';
import { buildTracks } from '../vp';
import { buildVpModel, type VpRow } from '../vpModel';
import { Icon } from './bits';
import { usePhoneLayout } from './usePhoneLayout';
import { feel } from '../feel';
import { HoldStrips, type RowRelation } from './VpHoldings';
import { RivalsTable } from './RivalsTable';
import { SidebarObjective } from './EraObjectives';
/** Always-visible victory race (pillar 4): every empire's VP, Peace or War and public numbers at a glance (`RivalsTable`). "Score ledger" opens the sheet. */
export function VpTrack() {
  const phone = usePhoneLayout();
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const observer = useGame((s) => s.observer);
  const setObserverPerspective = useGame((s) => s.observerSetPerspective);
  const set = useGame((s) => s.set);
  const seat = useGame((s) => s.seat);
  const orders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
  const humans = useGame((s) => s.humans);
  // Wars declared on you in the turn that just resolved read "War · new" until the next one.
  const newWars = useGame((s) => s.report?.alarms?.[s.seat]);
  const panel = useRef<HTMLElement>(null);
  const model = useMemo(() => (view ? buildVpModel(view) : null), [view]);
  if (!view) return null;
  const finished = view.finished || view.winner !== null;
  const tracks = buildTracks(view, { aliveOnly: true });
  // Incoming offers still awaiting a valid response: the same list as "Respond to offer".
  const incoming = pendingDecisions(view, orders[seat] ?? [], errors[seat] ?? [], { humans, orders, errors })
    .filter((d) => d.kind === 'diplomacy').length;
  // This seat's valid, still-pending declaration (the city card shows the same "War declared").
  const declaredOn = (team: number) => relationWith(view, team)?.state !== 'War' && (orders[seat] ?? []).some((o, i) =>
    o.type === 'DeclareWar' && o.against === team && !(errors[seat] ?? []).some((e) => e.index === i));
  const mine = tracks.find((t) => t.isMine);
  const leader = tracks[0];
  const nearRival = nearestRival(view);
  // A seat's relation with a team, as its row shows it; a declaration queued this turn reads as on the city card.
  const relationOf = (row: VpRow): RowRelation | null => {
    if (observer.active && observer.perspective === 'god') return null;
    if (row.isMine) return null;
    if (declaredOn(row.team)) return { kind: 'war', declared: true, text: 'War declared', title: 'Your declaration of War, pending until the turn resolves' };
    const relation = relationWith(view, row.team);
    const state = relation?.state ?? 'War';
    const isNew = state === 'War' && !!newWars?.some((a) => a.kind === 'war' && a.team === row.team);
    const guarantee = !relation?.security && (relation?.guarantee_until ?? 0) > view.turn ? ` ≤${relation!.guarantee_until - 1}` : '';
    return {
      kind: state.toLowerCase() as RowRelation['kind'], isNew,
      text: `${state}${isNew ? ' · new' : ''}${guarantee}`,
      title: relation?.security ? `Alliance protected until turn ${relation.security.protected_until}` : relation?.guarantee_until ? `Peace guaranteed through turn ${relation.guarantee_until - 1}` : state,
    };
  };
  const onRowClick = (team: number) => {
    if (observer.active) {
      const row = model!.rows.find((r) => r.team === team);
      const target = row?.members[0]?.id;
      if (target !== undefined) {
        setObserverPerspective(observer.perspective === target ? 'god' : target);
      }
      return;
    }
    set({ diplomacyOpen: true, diplomacyTeam: team });
  };
  const raceDetails = <>
      <header className="ph vhc-head">
        <h3>Victory race</h3>
        <button className="link" aria-haspopup="dialog" onClick={() => set({ vpExpanded: true })}>Score ledger</button>
      </header>
      {!finished && model!.holds.length > 0 && <div className="race-headline">
        <HoldStrips model={model!} named />
      </div>}
      {!finished && !observer.active && nearRival && <details className="sidebar-alerts">
        <summary><Icon name="ui-alert" size={14} />{rivalLine(view, nearRival).replace(/\.$/, '')}</summary>
        <RivalWin view={view} rival={nearRival} showSummary={false} />
      </details>}
      <RivalsTable
        view={view}
        model={model!}
        relationOf={relationOf}
        onOpen={onRowClick}
        observerPerspective={observer.active ? observer.perspective : undefined}
      />
      {!finished && !observer.active && <footer className="vp-foot rivals-foot">
        <button className="link" onClick={() => set({ diplomacyOpen: true })}>Diplomacy{incoming ? ` (${incoming})` : ''}</button>
      </footer>}
      {!finished && !observer.active && model && <SidebarObjective view={view} model={model} />}
  </>;
  return (
    <>
    {phone && <button className="phone-vp-strip" data-overlay-slot="victory-race" aria-label={`Victory race: you ${mine?.vp ?? 0} of ${mine?.target ?? 0} VP; leader ${leader?.vp ?? 0} of ${leader?.target ?? 0} VP`}
      aria-haspopup="dialog" onClick={() => { set({ vpExpanded: true }); feel.select(); }}>
      <span>You <b data-celebration-counter="vp">{mine?.vp ?? 0}/{mine?.target ?? 0}</b></span>
      <span>Lead {leader?.vp ?? 0}/{leader?.target ?? 0} <span aria-hidden="true">▾</span></span>
    </button>}
    {!phone && <section ref={panel} className="panel vp" aria-label="Victory race" data-overlay-slot="victory-race" style={{ ['--vp-teams' as string]: tracks.length }}>
      {raceDetails}
    </section>}
    </>
  );
}
