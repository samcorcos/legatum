import { useMemo, useRef } from 'react';
import { relationWith } from '../diplomacy';
import { nearestRival, rivalLine } from '../rivalWin';
import { RivalWin } from './RivalWin';
import { pendingDecisions } from '../turnFlow';
import { CounterDelta } from '../celebrate/MomentLane';
import { useGame } from '../store';
import { buildTracks } from '../vp';
import { buildVpModel } from '../vpModel';
import { Icon } from './bits';
import { usePhoneLayout } from './usePhoneLayout';
import { feel } from '../feel';
import { empireOfPlayer } from '../empires';
import { CAPITALS_RULE, CapitalChip, ClosedChip, DecadenceChip, HeldToken, HoldStrips, NobodyRow, LeadChip, PendingGoal, RowWho, TokenCard, tokenKey, useTokenCards, type RowRelation } from './VpHoldings';
import { empireVars } from './vpParts';
import { SidebarObjective } from './EraObjectives';
/** Always-visible victory race (pillar 4): what each empire holds, and what nobody does. "Score ledger" opens the sheet. */
export function VpTrack() {
  const phone = usePhoneLayout();
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const set = useGame((s) => s.set);
  const seat = useGame((s) => s.seat);
  const orders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
  const humans = useGame((s) => s.humans);
  // Wars declared on you in the turn that just resolved read "War · new" until the next one.
  const newWars = useGame((s) => s.report?.alarms?.[s.seat]);
  const panel = useRef<HTMLElement>(null);
  const model = useMemo(() => (view ? buildVpModel(view) : null), [view]);
  const cards = useTokenCards(model, panel);
  if (!view) return null;
  const finished = view.finished || view.winner !== null;
  const tracks = buildTracks(view, { aliveOnly: true });
  // Incoming offers still awaiting a valid response: the same list as "Respond to offer".
  const incoming = pendingDecisions(view, orders[seat] ?? [], errors[seat] ?? [], { humans, orders, errors })
    .filter((d) => d.kind === 'diplomacy').length;
  // Players are named as everywhere else: "You", or the empire (`empireNameOf`), never a raw seat name.
  const name = (id: number | null) => (id === null ? '—' : id === view.me ? 'You' : empireOfPlayer(view, id));
  // This seat's valid, still-pending declaration (the city card shows the same "War declared").
  const declaredOn = (team: number) => relationWith(view, team)?.state !== 'War' && (orders[seat] ?? []).some((o, i) =>
    o.type === 'DeclareWar' && o.against === team && !(errors[seat] ?? []).some((e) => e.index === i));
  const mine = tracks.find((t) => t.isMine);
  const leader = tracks[0];
  const rival = model!.rows.find(r => !r.isMine);
  const ownHold = model!.holds[0]?.isMine;
  const nearRival = nearestRival(view);
  const raceDetails = <>
      <header className="ph vhc-head">
        <h3>Victory race</h3>
        <button className="link" aria-haspopup="dialog" onClick={() => set({ vpExpanded: true })}>Score ledger</button>
      </header>
      <div className="race-headline" data-celebration-counter={!model!.holds.length || ownHold ? 'vp' : undefined}>
        {!finished && model!.holds.length > 0 ? <HoldStrips model={{ ...model!, holds: model!.holds.slice(0, 1) }} named /> : <div className="race-score"><Icon name="vp" size={26} /><span>{finished ? 'Final standing' : 'Your standing'}<b>{mine?.vp ?? 0} / {mine?.target ?? 10} VP</b></span></div>}
        {(!model!.holds.length || ownHold) && <CounterDelta counter="vp" />}
      </div>
      {!finished && nearRival && <details className="sidebar-alerts">
        <summary><Icon name="ui-alert" size={14} />{rivalLine(view, nearRival).replace(/\.$/, '')}</summary>
        <RivalWin view={view} rival={nearRival} />
      </details>}
      {!finished && model && <SidebarObjective view={view} model={model} />}
      <details className="sidebar-standings" onToggle={e => { if (!e.currentTarget.open) cards.reset(); }}>
        <summary data-celebration-counter={model!.holds.length && !ownHold ? 'vp' : undefined}>{model!.holds.length && !ownHold ? `You · ${mine?.vp ?? 0}/${mine?.target ?? 10} VP` : rival ? `${rival.label} · ${rival.vp} VP` : 'Standings'}{!!model!.holds.length && !ownHold && <CounterDelta counter="vp" />} <span>All empires</span></summary>
        {!finished && <span className="vhc-key" aria-hidden="true" title="Kept for good; held VP can be lost with the hold"><span className="vhc-tok demo" />Kept<span className="vhc-tok live demo" />Held</span>}
      {!finished && model!.holds.length > 1 && <HoldStrips model={{ ...model!, holds: model!.holds.slice(1) }} named />}
      {!finished && <p className="vhc-rule">{CAPITALS_RULE}</p>}
      <div className="vp-rows">
        {model!.rows.map((row) => {
          const relation = relationWith(view, row.team);
          const rel: RowRelation | null = row.isMine ? null : declaredOn(row.team)
            // A declaration queued this turn reads as on the city card: one rule, one wording.
            ? { kind: 'war', declared: true, text: 'War declared', title: 'Your declaration of War, pending until the turn resolves' }
            : (() => {
              const state = relation?.state ?? 'War';
              const isNew = state === 'War' && !!newWars?.some((a) => a.kind === 'war' && a.team === row.team);
              const guarantee = !relation?.security && (relation?.guarantee_until ?? 0) > view.turn ? ` ≤${relation!.guarantee_until - 1}` : '';
              return {
                kind: state.toLowerCase() as RowRelation['kind'], isNew,
                text: `${state}${isNew ? ' · new' : ''}${guarantee}`,
                title: relation?.security ? `Alliance protected until turn ${relation.security.protected_until}` : relation?.guarantee_until ? `Peace guaranteed through turn ${relation.guarantee_until - 1}` : state,
              };
            })();
          return (
          <div key={row.team} className={`vp-row vhc-row${row.isMine ? ' mine' : ''}${row.hold ? ' holding' : ''}`} style={empireVars(row.members[0])} role="group" aria-label={`${row.label}: ${row.vp} of ${row.target} VP`}>
            <RowWho members={row.members} relation={rel} nameOf={(e) => name(e.id)} onOpen={() => set({ diplomacyOpen: true, diplomacyTeam: row.team })} />
            <span className="vp-label"><span className="vp-team-name">{row.label}</span>{row.members.some((e) => e.aiLevel) && <small className="vp-ai-level">{row.members.filter((e) => e.aiLevel).map((e) => row.members.length === 1 ? `AI · ${e.aiLevel}` : `${e.name}: ${e.aiLevel}`).join(' · ')}</small>}</span>
            <span className="vhc-holds">
              {row.tokens.map((t) => <HeldToken key={tokenKey(t)} t={t} props={cards.tokenProps(tokenKey(t), t.source.id, t.owner.id)} />)}
              <LeadChip members={row.members} rules={model!.raceRules.rules} />
              <DecadenceChip members={row.members} rules={model!.raceRules.rules} />
              <ClosedChip members={row.members} />
              <CapitalChip row={row} nameOf={(e) => name(e.id)} />
              {row.pendingGoals.map((t) => <PendingGoal key={tokenKey(t)} t={t} props={cards.tokenProps(tokenKey(t), t.source.id, t.owner.id)} />)}
            </span>
            <span className="vp-num">
              <b>{row.vp}</b>/{row.target}
            </span>
          </div>
          );
        })}
      </div>
      {!finished && model && <NobodyRow model={model} tokenProps={cards.tokenProps} />}
      {!finished && <footer className="vp-foot">
        <button className="link" onClick={() => set({ diplomacyOpen: true })}>Diplomacy{incoming ? ` (${incoming})` : ''}</button>
      </footer>}
      </details>
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
    {!phone && <TokenCard model={model} active={cards.active} panel={panel} />}
    </>
  );
}
