// The score ledger (round 3 of "Every Way to Score", design mockup): one table answers the four
// questions a player opens this screen with. Who's winning: the empires are the columns, in rank
// order, with their VP. How close am I: your column is gilt wherever it falls. What's my next point:
// every way to score is a row, and your cell says held, open or not reached, with your progress
// where only you can see it. What can I take: the world wonder rows show who builds and holds each. A row expands in place for its rule and the public facts behind it; nothing here
// recommends a route (user, 2026-10-06).
import { Fragment, useEffect, useRef, useState, type ReactNode } from 'react';
import { useGame } from '../store';
import { Icon } from './bits';
import { Amount } from './Amounts';
import { Glyph } from './ringMedallion';
import { CrestMedallion, Star, roman } from '../brand/ornaments';
import { PortraitRoundel } from './premium';
import { EraObjectives, FAMILY_ICON, Meter, VpSeal, empireVars } from './vpParts';
import { holdWords } from './VpHoldings';
import type { Source, VpEmpire, VpModel, VpRow } from '../vpModel';
import type { ObjectiveFamily } from '../types';
import './scoreLedger.css';

const fmt = (n: number) => n.toLocaleString('en-US');

/** A point marker: the empire's ink and the source's icon. */
function Token({ e, icon, of, title, dim }: { e: VpEmpire; icon: string; of?: VpEmpire; title?: string; dim?: boolean }) {
  return <span className={`sl-token${e.isMe ? ' mine' : ''}${dim ? ' dim' : ''}`} style={empireVars(e)} title={title} role="img" aria-label={title}>
    <Glyph name={icon} size={13} />
    {of && of.id !== e.id && <span className="sl-token-of"><CrestMedallion faction={of.faction} color={of.ink} size={11} /></span>}
  </span>;
}

function Socket({ title, faint }: { title?: string; faint?: boolean }) {
  return <span className={`sl-socket${faint ? ' faint' : ''}`} title={title} role="img" aria-label={title} />;
}

function WorkCell({ s, e }: { s: Source; e: VpEmpire }) {
  const f = s.facts.find((x) => x.id === e.id);
  if (s.owners.includes(e.id)) return <div className="sl-cell"><Token e={e} icon={s.icon} title={`${e.isMe ? 'You' : e.name}: ${f?.label ?? 'world wonder'} · +${s.vp} VP`} /><small className="sl-under">{f?.label}</small></div>;
  if (f?.label) {
    const [name, prog] = f.label.split(/ (?=\d)/);
    return <div className="sl-cell"><Socket title={`${e.isMe ? 'You are' : `${e.name} is`} building the ${f.label}`} /><small className="sl-under">{name}<span className="sl-building"> {prog}</span></small></div>;
  }
  return <div className="sl-cell"><Socket title={`${e.isMe ? 'You have' : `${e.name} has`} no world wonder`} /></div>;
}

/** The faction goal row: each empire's own goal (and any held goal its colours score for). */
function GoalCell({ model, e, held }: { model: VpModel; e: VpEmpire; held: boolean }) {
  const scoring = e.held.filter((s) => s.kind === 'goal' && Boolean(s.live || s.held) === held);
  const g = e.goal;
  const ownGoalHere = Boolean(g.live || g.held) === held;
  if (!ownGoalHere && !scoring.length) return <div className="sl-cell"><span className="sl-empty">—</span></div>;
  const of = (s: Source) => model.empires.find((x) => x.id === s.only);
  if (scoring.length) {
    const ownScored = scoring.some((s) => s.id === g.id);
    return <div className="sl-cell">
      <span className="sl-tokens">{scoring.map((s) => <Token key={s.id} e={e} icon={s.icon} of={of(s)} title={`${e.isMe ? 'You' : e.name}: ${s.name} · +${s.vp} VP${s.live || s.held ? ' · held, can be lost' : ''}`} />)}{ownGoalHere && !ownScored && <Socket title={`${e.isMe ? 'Your' : `${e.name}’s`} goal: ${g.name}. ${g.rule}. ${g.holdNote ?? 'Not yet scored'}`} />}</span>
      <small className="sl-under">{scoring.map((s) => s.name).join(', ')}</small>
      {e.isMe && ownGoalHere && !ownScored && g.progress && <span className="sl-mini-meter" role="progressbar" aria-valuemin={0} aria-valuemax={g.progress.target} aria-valuenow={Math.min(g.progress.value, g.progress.target)} aria-label={g.progress.label}><i style={{ width: `${Math.min(100, (g.progress.value / Math.max(1, g.progress.target)) * 100)}%` }} /><b>{fmt(Math.min(g.progress.value, g.progress.target))}/{fmt(g.progress.target)}</b></span>}
    </div>;
  }
  const publicLine = g.facts[0]?.label;
  return <div className="sl-cell" title={`${e.isMe ? 'Your goal' : `${e.name}’s goal`}: ${g.name}. ${g.rule}`}>
    <Socket title={`${e.isMe ? 'Your' : `${e.name}’s`} goal: ${g.name}. ${g.rule}. ${g.holdNote ?? 'Not yet scored'}`} />
    <small className="sl-under">{g.name}</small>
    {e.isMe && g.progress && <span className="sl-mini-meter" role="progressbar" aria-valuemin={0} aria-valuemax={g.progress.target} aria-valuenow={Math.min(g.progress.value, g.progress.target)} aria-label={g.progress.label}><i style={{ width: `${Math.min(100, (g.progress.value / Math.max(1, g.progress.target)) * 100)}%` }} /><b>{fmt(Math.min(g.progress.value, g.progress.target))}/{fmt(g.progress.target)}</b></span>}
    {!e.isMe && publicLine && <small className="sl-public">{publicLine.replace(/ \(public count\)$/, '')}</small>}
  </div>;
}

/** The capitals row: one token per rival first capital this empire holds now, in its ink. */
function CapitalCell({ model, e }: { model: VpModel; e: VpEmpire }) {
  const names = model.capitals.find((c) => c.owner === e.id)?.names ?? [];
  const who = e.isMe ? 'You' : e.name;
  if (!names.length) return <div className="sl-cell"><Socket title={`${who} ${e.isMe ? 'hold' : 'holds'} no rival's first capital`} /><small className="sl-under">None held</small></div>;
  return <div className="sl-cell">
    <span className="sl-tokens">{names.map((n) => <Token key={n} e={e} icon="city" title={`${who} ${e.isMe ? 'hold' : 'holds'} ${n} · 1 VP while ${e.isMe ? 'you hold' : 'it holds'} it`} />)}</span>
    <small className="sl-under">{names.join(', ')}</small>
  </div>;
}

function EraCell({ s, e }: { s: Source; e: VpEmpire }) {
  const info = s.era!;
  const who = e.isMe ? 'You' : e.name;
  // One check for the token and the row's status word: has this empire scored the Age (`info.scored`)?
  // The card it used names the token; a save whose card is no longer drawn gets the Age's own mark.
  if (!info.scored.includes(e.id)) return <div className="sl-cell"><Socket title={`${who} ${e.isMe ? 'have' : 'has'} not scored the ${info.eraName} Age yet`} /></div>;
  const scoredWith = info.objectives.find((o) => o.scorers.includes(e.id));
  if (!scoredWith) return <div className="sl-cell"><Token e={e} icon={s.icon} title={`${who} scored the ${info.eraName} Age`} /></div>;
  return <div className="sl-cell"><Token e={e} icon={FAMILY_ICON[scoredWith.family as ObjectiveFamily]} title={`${who} scored the ${info.eraName} Age with: ${scoredWith.text}`} /></div>;
}

/** An Age's status for the viewer, in one line. */
function eraLine(s: Source): string {
  return s.era!.myScored ? 'You scored this Age.' : 'Every empire sees this card the turn the first empire enters the era, and it never expires.';
}

function RowDetail({ model, s, kind }: { model: VpModel; s?: Source; kind: 'work' | 'goal' | 'heldGoal' | 'capital' | 'era' }) {
  if (kind === 'goal' || kind === 'heldGoal') {
    return <div className="sl-detail">
      <p className="sl-detail-rule">{kind === 'heldGoal' ? 'These faction goals score 2 VP while their condition or item is held; they can be lost.' : 'These faction goals score 2 VP once and are kept for good.'}</p>
      <ul className="sl-goals">
        {model.empires.filter((e) => Boolean(e.goal.live || e.goal.held) === (kind === 'heldGoal')).map((e) => {
          const g = e.goal;
          const done = g.owners.length > 0;
          return <li key={e.id} className={e.isMe ? 'me' : ''} style={empireVars(e)}>
            <CrestMedallion faction={e.faction} color={e.ink} size={18} />
            <div>
              <span className="sl-goal-title"><b>{e.isMe ? 'Your goal' : e.name}: {g.name}</b><VpSeal vp={g.vp} className="inline sl-seal" /></span>
              <span className="sl-goal-rule">{g.rule}.{g.live && !g.held ? ' Lost the turn it stops holding.' : ''}</span>
              {g.held && g.holdNote && <span className="sl-goal-note">{g.holdNote}</span>}
              {g.heldAt && <button type="button" className="btn quiet sl-locate" onClick={() => { useGame.getState().set({ vpExpanded: false }); useGame.getState().jumpTo(g.heldAt!); }}>Show on map</button>}
              {e.isMe && g.progress && !done && <Meter p={g.progress} color={e.ink} />}
            </div>
            <span className={`sl-goal-state${done ? ' done' : ''}`}>{done ? (g.held ? `Held by ${model.empires.find((x) => x.id === g.owners[0])?.name ?? 'another empire'}` : 'Done') : 'Not yet'}</span>
          </li>;
        })}
      </ul>
    </div>;
  }
  if (kind === 'capital') {
    return <div className="sl-detail">
      <p className="sl-detail-rule">Hold a rival's first capital: 1 VP while you hold it. You lose the point if anyone takes the city or you raze it. The Celts keep the point of a capital they sack.</p>
      <ul className="sl-goals">
        {model.empires.map((e) => {
          const names = model.capitals.find((c) => c.owner === e.id)?.names ?? [];
          return <li key={e.id} className={e.isMe ? 'me' : ''} style={empireVars(e)}>
            <CrestMedallion faction={e.faction} color={e.ink} size={18} />
            <div><span className="sl-goal-title"><b>{e.isMe ? 'You' : e.name}</b></span><span className="sl-goal-rule">{names.length ? names.join(', ') : 'No rival capital held'}</span></div>
            <span className={`sl-goal-state${names.length ? ' done' : ''}`}>{names.length} VP</span>
          </li>;
        })}
      </ul>
    </div>;
  }
  if (!s) return null;
  if (kind === 'work') {
    const building = s.facts.filter((f) => f.label && !s.owners.includes(f.id));
    return <div className="sl-detail">
      <p className="sl-detail-rule">{s.rule}</p>
      <p className="sl-detail-note">Each world wonder can be completed only once. Its tile holder scores 2 VP. Capture the city or claim the tile to take those points.{building.length ? ` Known builders: ${building.map((f) => `${model.empires.find((e) => e.id === f.id)?.name} (${f.label})`).join(', ')}.` : ' No known wonder is under construction.'}</p>
    </div>;
  }
  const info = s.era!;
  // The capital and wonder exception belongs only to the two capture cards.
  const captureCard = info.objectives.some((o) => o.kind === 'HumbleTheMighty' || o.kind === 'Breach');
  return <div className={`sl-detail sl-era${info.myScored ? ' scored' : ''}`}>
    <p className="sl-detail-rule">{eraLine(s)} <span className="sl-detail-fine">{`${info.myScored ? 'Every empire sees this card the turn the first empire enters the era, and it never expires. ' : ''}Each card counts only what you do after its Age begins.${captureCard ? ' Humble the mighty and Breach never count capitals or wonder cities.' : ''}`}</span></p>
    <EraObjectives model={model} s={s} />
  </div>;
}

type Row = { id: string; kind: 'work' | 'goal' | 'heldGoal' | 'capital' | 'era'; name: ReactNode; rule: string; vp: number; s?: Source; numeral?: string; dim?: boolean; status?: string };

/** The viewer's standing in an Age, as one or two words beside its name. */
function eraStatusWord(s: Source): string {
  return s.era!.myScored ? 'scored' : 'open to every empire';
}

function rowsOf(model: VpModel): { kept: Row[]; held: Row[]; eras: Row[] } {
  const kept: Row[] = [
    ...model.works.filter((s) => !s.live && !s.held).map((s): Row => ({ id: s.id, kind: 'work', name: s.name, rule: 'Complete a world wonder', vp: s.vp, s })),
    { id: 'goal', kind: 'goal', name: 'Faction goals', rule: 'Kept once scored', vp: 2 },
  ];
  const held: Row[] = [
    ...model.works.filter((s) => s.live || s.held).map((s): Row => ({ id: s.id, kind: 'work', name: s.name, rule: s.rule, vp: s.vp, s })),
    { id: 'held-goal', kind: 'heldGoal', name: 'Faction goals', rule: 'Lost when the condition or item is lost', vp: 2 },
    { id: 'capitals-held', kind: 'capital', name: 'Rival capitals', rule: "Hold a rival's first capital: 1 VP while you hold it", vp: 1 },
  ];
  const eras: Row[] = model.eras.map((s): Row => ({
    id: s.id, kind: 'era', name: s.era!.eraName, numeral: roman(s.era!.era), rule: '', vp: s.vp, s,
    dim: s.era!.entered.length === 0 && s.owners.length === 0, status: eraStatusWord(s),
  }));
  return { kept, held, eras };
}

/** The column head: the empire, its VP, a 10-pip track, the hold countdown and decadence when they apply. */
function ColumnHead({ row, e, span, showDecadence }: { row: VpRow; e: VpEmpire; span: number; showDecadence: boolean }) {
  const first = row.members[0].id === e.id;
  const shortName = e.name.replace(/^Switzerland · AI /, 'Swiss · AI ');
  return <th scope="col" className={`sl-col${e.isMe ? ' mine' : ''}${first ? ' first' : ''}${span > 1 ? ' team' : ''}${row.isMine && span > 1 ? ' own-team' : ''}`} style={empireVars(e)}>
    <span className="sl-rank" aria-label={`Rank ${row.rank}`}>{first ? roman(row.rank) : ''}</span>
    <PortraitRoundel faction={e.faction} size={30} color={e.ink} />
    <span className="sl-col-name" title={e.name}>{e.isMe ? 'You' : shortName}</span>
    {e.aiLevel && <small className="sl-ai-level">AI · {e.aiLevel}</small>}
    <span className="sl-col-vp"><b>{row.vp}</b><small>/{row.target} VP</small></span>
    <span className="sl-gold" title={`${e.isMe ? 'Your' : 'Its'} treasury: every empire's Gold is public`}><Amount n={e.gold} unit="Gold" /></span>
    {first && <span className="sl-pips" role="img" aria-label={`${row.vp} of ${row.target} VP`}>{Array.from({ length: row.target }, (_, i) => <i key={i} className={i < row.vp ? 'on' : ''} />)}</span>}
    {first && row.hold && <span className={`sl-hold${row.isMine ? ' mine' : ''}`} title={holdWords(row)}>{row.isMine ? (span > 1 ? 'Your team wins in' : 'You win in') : 'Wins in'} {row.hold.left}</span>}
    {showDecadence && e.decadence > 0 && <span className="sl-decadence" title={`Power ${e.power}; ${e.lead}% above the average empire. Decadence slows city growth ${e.decadence}% and cuts Production, Gold and Science ${e.decadence}%.`}>decadence −{e.decadence}%</span>}
  </th>;
}

export function ScoreLedger({ model, turn, close, closeRef }: { model: VpModel; turn: number; close: () => void; closeRef: React.RefObject<HTMLButtonElement | null> }) {
  // The Age in focus is the newest one you have not scored: every empire can score it from the turn it begins. It opens
  // with the ledger, so its card is in view.
  const current = [...model.eras].reverse().find((s) => !s.era!.myScored);
  const [open, setOpen] = useState<string | null>(() => current?.id ?? null);
  const toggle = (id: string) => setOpen((o) => (o === id ? null : id));
  const body = useRef<HTMLDivElement>(null);
  // The ledger opens on the open Age, so its cards and claims are in view, not below the held rows.
  useEffect(() => {
    if (!current) return;
    // The sheet's dialog opens in its own effect, which runs after this one: until then the body has no layout, so wait a frame.
    const frame = requestAnimationFrame(() => {
      const box = body.current;
      const row = box?.querySelector<HTMLElement>(`[data-row="${current.id}"]`);
      if (!box || !row) return;
      // Scroll so the Ages heading is the first thing under the column heads.
      const sections = [...box.querySelectorAll<HTMLElement>('tr.sl-section')].filter((t) => t.offsetTop <= row.offsetTop);
      const top = sections.length ? sections[sections.length - 1].offsetTop : row.offsetTop;
      box.scrollTop = Math.max(0, top - (box.querySelector('thead')?.getBoundingClientRect().height ?? 0));
    });
    return () => cancelAnimationFrame(frame);
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
  const columns = model.rows.flatMap((row) => row.members.map((e) => ({ row, e })));
  const n = columns.length;
  const { kept, held, eras } = rowsOf(model);
  const target = model.rows[0]?.target ?? 10;

  const renderRow = (r: Row) => {
    const isOpen = open === r.id;
    const cells = columns.map(({ e }) => {
      let cell: ReactNode;
      if (r.kind === 'work') cell = <WorkCell s={r.s!} e={e} />;
      else if (r.kind === 'capital') cell = <CapitalCell model={model} e={e} />;
      else if (r.kind === 'goal' || r.kind === 'heldGoal') cell = <GoalCell model={model} e={e} held={r.kind === 'heldGoal'} />;
      else cell = <EraCell s={r.s!} e={e} />;
      return <td key={e.id} className={`${e.isMe ? 'mine' : ''}${e.id === columns[0].e.id ? ' first' : ''}`} style={empireVars(e)}>{cell}</td>;
    });
    return <Fragment key={r.id}>
      <tr key={r.id} className={`sl-row k-${r.kind}${isOpen ? ' open' : ''}${r.dim ? ' dim' : ''}`} data-row={r.id}>
        <th scope="row" className="sl-label">
          <button type="button" className="sl-row-btn" aria-expanded={isOpen} aria-controls={`sl-detail-${r.id}`} onClick={() => toggle(r.id)} title={r.rule || undefined}>
            <span className="sl-chev" aria-hidden="true"><i className="sl-chev-fallback" /></span>
            {r.numeral && <span className="sl-numeral">{r.numeral}</span>}
            <span className="sl-label-text">
              <span className="sl-name">{r.name} <VpSeal vp={r.vp} className="inline sl-seal" />{r.status && <span className="sl-status">{r.status}</span>}</span>
              {r.rule && <span className="sl-rule">{r.rule}</span>}
            </span>
          </button>
        </th>
        {cells}
      </tr>
      {isOpen && <tr key={`${r.id}-detail`} className="sl-row-detail" id={`sl-detail-${r.id}`}><td colSpan={n + 1}><RowDetail model={model} s={r.s} kind={r.kind} /></td></tr>}
    </Fragment>;
  };

  const section = (title: string, sub: string, icon: string) => <tr className="sl-section"><th scope="colgroup" colSpan={n + 1}><span className="sl-sec"><Icon name={icon} size={11} />{title}<span className="sl-sec-sub">{sub}</span></span><span className="sl-sec-rule" /></th></tr>;

  return <>
    <header className="sl-head">
      <div>
        <span className="sl-kick"><Star size={8} /> Victory points · turn {turn}</span>
        <h2 id="vh-sheet-title">Score ledger</h2>
      </div>
      <p className="sl-win">The first team to hold <b>{target} VP</b> for 10 turns wins. World wonders, rival capitals and goals can be lost; Age points stay earned.</p>
      <button ref={closeRef} type="button" className="btn quiet vh-close" onClick={close} aria-label="Close objectives">Close <kbd className="kbd">Esc</kbd></button>
    </header>
    <div className="sl-body" ref={body}>
      <table className="sl-table" style={{ ['--cols' as string]: n }}>
        <thead>
          <tr className="sl-cols">
            <th scope="col" className="sl-corner"><span className="sl-corner-text">How to score</span><span className="sl-corner-sub">The race, best first. Your column is gilt.</span></th>
            {columns.map(({ row, e }) => <ColumnHead key={e.id} row={row} e={e} span={row.members.length} showDecadence={model.raceRules.rules.decadenceSlope > 0} />)}
          </tr>
        </thead>
        <tbody>
          {section('Kept for good', 'Faction goals +2 each', 'vp')}
          {kept.map(renderRow)}
          {section('Held', 'World wonders and goals +2 each · rival capitals +1 each · you lose these if you lose the city, tile or condition', 'vp')}
          {held.map(renderRow)}
          {section('Ages', '+1 each, kept for good · score an Age with its one card, any time after it begins', 'milestone-six-techs')}
          {eras.map(renderRow)}
        </tbody>
      </table>
    </div>
  </>;
}
