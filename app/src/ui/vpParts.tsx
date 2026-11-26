// The shared pieces of the victory UI: the wax VP seal, an empire's control marker and the empty
// socket, an Age's card as a list, and the progress meter. The Score ledger and the score check use them.
import type { CSSProperties } from 'react';
import { Icon } from './bits';
import { CrestMedallion } from '../brand/ornaments';
import { ERA_COUNT } from '../data';
import { eraName } from './eraResearch';
import { cardMeter, scoredCard } from '../ageCards';
import { useGame } from '../store';
import { type EraObjective, type Progress, type Source, type VpEmpire, type VpModel } from '../vpModel';
import type { ObjectiveFamily } from '../types';
import './vpParts.css';

const css = (v: Record<string, string | number>) => v as CSSProperties;

/** The empire's colours as CSS variables: `--oc` for fills and tints, `--oi` for outlines and bars on paper. */
export const empireVars = (e?: Pick<VpEmpire, 'color' | 'ink'>): CSSProperties => css({ '--oc': e?.color ?? 'var(--ink-2)', '--oi': e?.ink ?? 'var(--ink-2)' });

/** The VP value as a small wax seal: "+2" over the VP mark. */
export function VpSeal({ vp, className }: { vp: number; className?: string }) {
  return <span className={`vh-seal ${className ?? ''}`} role="img" aria-label={`${vp} VP`}><b>+{vp}</b><Icon name="vp" size={8} /></span>;
}

/** An owner's control marker: their crest on a disc in their colour (TI4's control token). */
export function Marker({ e, size = 20, title }: { e: VpEmpire; size?: number; title?: string }) {
  return <span className={`vh-marker${e.isMine ? ' mine' : ''}`} style={{ ...(e.unmet ? {} : empireVars(e)), ['--ms' as string]: `${size}px` }} title={title ?? e.name}>
    <EmpireCrest e={e} size={size} />
  </span>;
}

/** An empire's crest, or a dashed "?" for one this seat has not met: its faction stays hidden. */
export function EmpireCrest({ e, size }: { e: Pick<VpEmpire, 'faction' | 'ink' | 'unmet'>; size: number }) {
  return e.unmet
    ? <span className="vh-unknown" style={{ ['--us' as string]: `${size}px` }} aria-hidden="true">?</span>
    : <CrestMedallion faction={e.faction} color={e.ink} size={size} />;
}

/** An empty socket: nobody's marker here yet. */
export function Socket({ size = 20, e }: { size?: number; e?: VpEmpire }) {
  // With an empire: its crest, faded until earned, so an empty circle is never ownerless.
  return <span className={`vh-socket${e ? ' owned' : ''}`} style={{ ...(e && !e.unmet ? empireVars(e) : {}), ['--ms' as string]: `${size}px` }} title={e ? `${e.name}: not yet` : undefined}>
    {e && <EmpireCrest e={e} size={Math.round(size * 0.8)} />}
  </span>;
}

export const FAMILY_ICON: Record<ObjectiveFamily, string> = { Build: 'building-workshop', Expand: 'city', Fight: 'unit-spearman' };
const COUNT_UNIT: Record<string, string> = { Cities: 'cities', Population: 'population', Territory: 'tiles', Buildings: 'buildings' };

/** An Age's card: its family, the rule, your own progress, and who has scored the Age with it. */
function EraObjectiveRow({ model, o, scored, era }: { model: VpModel; o: EraObjective; scored: boolean; era: number }) {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const by = o.scorers.map((id) => model.empires.find((e) => e.id === id)).filter((e): e is VpEmpire => !!e);
  const me = model.empires.find((e) => e.isMe);
  const mineDone = o.scorers.includes(model.me);
  const have = o.mine;
  // An Age you scored with another card has nothing left to count on this one: no meter.
  const idle = scored && !mineDone;
  const rule = o.objective;
  const meter = have && !idle && (rule && cardMeter(rule, have.current, have.target)
    || { value: have.current, target: have.target, label: have.current >= have.target ? (scored ? 'Met' : 'Met: scores at turn end') : `${have.current} of ${have.target}` });
  const turn = mineDone && view ? scoredCard(view, era)?.turn : undefined;
  const ownCount = !['Breach', 'HumbleTheMighty', 'Veterans'].includes(o.kind ?? '');
  return <li className={`vh-obj fam-${o.family.toLowerCase()}${mineDone ? ' mine' : ''}${by.length && !mineDone ? ' theirs' : ''}${idle ? ' idle' : ''}`} data-objective={o.slot}>
    <span className="vh-obj-fam"><Icon name={FAMILY_ICON[o.family]} size={11} />{o.family}</span>
    <span className="vh-obj-text">{o.text}</span>
    {meter && <Meter p={meter} color={me?.ink} />}
    {have && !scored && ownCount && have.countBaseline != null && have.countTotal != null && have.countingSinceTurn != null &&
      <span className="vh-obj-account" data-testid="era-count-account">You had {have.countBaseline} {COUNT_UNIT[o.kind ?? ''] ?? 'in total'} on turn {have.countingSinceTurn}; now {have.countTotal}.</span>}
    {have && !scored && (have.ready ?? have.current >= have.target) && (have.restartEras?.length ?? 0) > 0 &&
      <span className="vh-obj-restart" data-testid="era-count-restart">When this scores, your {have.restartEras!.map((e) => eraName(e - 1)).join(' and ')} progress goes back to zero.</span>}
    <span className="vh-obj-by">
      {by.length ? by.map((e) => <Marker key={e.id} e={e} size={15} title={`${e.isMe ? 'You' : e.name} scored this Age with it`} />) : <Socket size={15} />}
      <small>{mineDone && turn !== undefined ? `Scored on turn ${turn} with this` : by.length ? 'Scored with this' : 'Nobody yet'}</small>
    </span>
  </li>;
}

/** An Age's card as a list (one card, shown full width); the card and the sheet both render it. */
export function EraObjectives({ model, s }: { model: VpModel; s: Source }) {
  const e = s.era;
  if (!e) return null;
  return <ul className="vh-objs single" aria-label={`${s.name}: card`}>
    {e.objectives.map((o) => <EraObjectiveRow key={o.slot} model={model} o={o} scored={e.myScored} era={e.era} />)}
    {e.era === Math.max(...model.eras.map((x) => x.era?.era ?? 0)) && e.era < ERA_COUNT && <li className="vh-obj-next">The next Age begins when any empire enters the {eraName(e.era)} era; its card stays hidden until then.</li>}
  </ul>;
}

export function Meter({ p, color }: { p: Progress; color?: string }) {
  return <div className="vh-meter" style={css({ '--oi': color ?? 'var(--ink-2)' })}>
    <span className="vh-meter-bar" role="progressbar" aria-label={p.label} aria-valuemin={0} aria-valuemax={p.target} aria-valuenow={Math.min(p.value, p.target)}><i style={{ width: `${Math.min(100, (p.value / Math.max(1, p.target)) * 100)}%` }} /></span>
    <span className="vh-meter-label">{p.label}</span>
  </div>;
}
