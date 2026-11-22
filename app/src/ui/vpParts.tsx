// The shared pieces of the victory UI: the wax VP seal, an empire's control marker and the empty
// socket, a source's medallion, and the objective card (a TI4 public objective). The race panel's
// token cards and the objective-cards sheet render the same card, so a source reads the same
// everywhere. Ownership has three looks from the viewer's side: gilt frame and star stud (yours),
// the holder's colour and crest (a rival's), a dashed frame and "Nobody yet" (nobody's).
import type { CSSProperties, ReactNode } from 'react';
import { Icon } from './bits';
import { RingMedallion } from './ringMedallion';
import { CrestMedallion, Star } from '../brand/ornaments';
import { ERA_COUNT, FACTION_GOAL } from '../data';
import { eraName } from './eraResearch';
import { cardMeter, scoredCard } from '../ageCards';
import { useGame } from '../store';
import { ownWords, type EraObjective, type Progress, type Source, type VpEmpire, type VpModel } from '../vpModel';
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
  return <span className={`vh-marker${e.isMine ? ' mine' : ''}`} style={{ ...empireVars(e), ['--ms' as string]: `${size}px` }} title={title ?? e.name}>
    <CrestMedallion faction={e.faction} color={e.ink} size={size} />
  </span>;
}

/** An empty socket: nobody's marker here yet. */
export function Socket({ size = 20, e }: { size?: number; e?: VpEmpire }) {
  // With an empire: its crest, faded until earned, so an empty circle is never ownerless.
  return <span className={`vh-socket${e ? ' owned' : ''}`} style={{ ...(e ? empireVars(e) : {}), ['--ms' as string]: `${size}px` }} title={e ? `${e.name}: not yet` : undefined}>
    {e && <CrestMedallion faction={e.faction} color={e.ink} size={Math.round(size * 0.8)} />}
  </span>;
}

const ownTone = (s: Source): 'gilt' | 'neutral' | 'hollow' => (s.own === 'mine' ? 'gilt' : s.own === 'theirs' ? 'neutral' : 'hollow');

/** The source's medallion: gilt when yours, ringed in the holder's colour with their crest when a rival's, hollow when open. */
export function SourceMedal({ model, s, size = 32 }: { model: Pick<VpModel, 'empires'>; s: Source; size?: number }) {
  const single = s.owners.length === 1 ? model.empires.find((e) => e.id === s.owners[0]) : undefined;
  return <span className={`vh-medal own-${s.own}`} style={empireVars(single)}>
    <RingMedallion icon={s.icon} tone={ownTone(s)} size={size}
      badge={single ? <Marker e={single} size={Math.max(14, Math.round(size * 0.5))} /> : s.own === 'mine' ? <Icon name="ui-check" size={10} className="ms-check" /> : undefined} />
  </span>;
}

export const FAMILY_ICON: Record<ObjectiveFamily, string> = { Build: 'building-workshop', Expand: 'city', Fight: 'unit-soldier' };
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

/** One marker socket per living empire: filled where it is earned, with the public fact under it. */
function ControlRow({ model, s, size = 20 }: { model: VpModel; s: Source; size?: number }) {
  return <div className="vh-ctrl">
    {model.empires.map((e) => {
      const has = s.owners.includes(e.id);
      const f = s.facts.find((x) => x.id === e.id);
      return <span key={e.id} className={`vh-ctrl-slot${e.isMe ? ' me' : ''}${has ? ' held-first' : ''}`}>
        {has ? <Marker e={e} size={size} /> : <Socket size={size} e={e} />}
        <small>{has || s.kind === 'work' ? '' : f?.label ?? ''}</small>
      </span>;
    })}
  </div>;
}

export function Meter({ p, color }: { p: Progress; color?: string }) {
  return <div className="vh-meter" style={css({ '--oi': color ?? 'var(--ink-2)' })}>
    <span className="vh-meter-bar" role="progressbar" aria-label={p.label} aria-valuemin={0} aria-valuemax={p.target} aria-valuenow={Math.min(p.value, p.target)}><i style={{ width: `${Math.min(100, (p.value / Math.max(1, p.target)) * 100)}%` }} /></span>
    <span className="vh-meter-label">{p.label}</span>
  </div>;
}

function WorkLines({ model, s }: { model: VpModel; s: Source }) {
  const building = s.facts.filter((f) => f.label && !s.owners.includes(f.id));
  if (!building.length) return <p className="vh-private">No world wonder under construction{s.owners.length ? ' elsewhere' : ''}</p>;
  return <div className="vh-work">{building.map((f) => {
    const e = model.empires.find((x) => x.id === f.id)!;
    return <div key={f.id} className="vh-work-row"><CrestMedallion faction={e.faction} color={e.ink} size={13} /><span>{f.label}</span></div>;
  })}</div>;
}

/** A live goal's loss rule, in the goal table's words. */
const liveWords = (faction: NonNullable<Source['faction']>) => {
  const note = FACTION_GOAL[faction].liveNote ?? '';
  return note ? `${note[0].toUpperCase()}${note.slice(1)}.` : '';
};

/** A held goal's public facts: whose item it is, who holds it now, and whether it can be placed on the map. */
function HeldLines({ model, s, goalOwner }: { model: VpModel; s: Source; goalOwner?: VpEmpire }) {
  const holder = s.owners.length === 1 ? model.empires.find((e) => e.id === s.owners[0]) : undefined;
  const hall = s.faction === 'Mongols';
  return <div className="vh-held" data-held-by={holder?.id}>
    {goalOwner && <div className="vh-held-row"><Marker e={goalOwner} size={16} title={goalOwner.name} /><span>{`${goalOwner.isMe ? 'Your' : `${goalOwner.name}’s`} ${hall ? 'Yam Relay Station' : 'Heir'}`}</span></div>}
    <div className={`vh-held-row now${holder ? '' : ' nobody'}`}>
      {holder ? <Marker e={holder} size={16} /> : <Socket size={16} />}
      <b>{s.holdNote}</b>
    </div>
    {s.heldAt && <p className="vh-private" data-testid="heir-locate-hint">{hall ? 'Its city is on your map: click its token in the race panel to show it.' : 'His position is public: click his token in the race panel to show him on the map.'}</p>}
    {holder && <p className="vh-private">{hall ? 'The points and the open road go with its city: a rival that captures the city scores them, and they return when it is retaken.' : 'The points go with him: a rival that captures him scores them, and they return when he is retaken.'}</p>}
  </div>;
}

/** The objective card: its band says whose it is, the markers or standings show who has it. */
export function SourceCard({ model, s, compact, terms = true, as: As = 'article', children }: {
  model: VpModel; s: Source; compact?: boolean; terms?: boolean; as?: 'article' | 'div'; children?: ReactNode;
}) {
  const single = s.owners.length === 1 ? model.empires.find((e) => e.id === s.owners[0]) : undefined;
  const goalOwner = s.only !== undefined ? model.empires.find((e) => e.id === s.only) : undefined;
  // A held goal's band is its holder's; any other goal's is its own empire's, once done.
  const bandOwner = s.kind === 'goal' && !s.held ? (s.owners.length ? goalOwner : undefined) : single;
  const tint = bandOwner ?? goalOwner;
  const sockets = model.empires.length > 4 ? (compact ? 14 : 18) : 20;
  return <As className={`vh-card k-${s.kind} own-${s.own}${s.live ? ' live' : ''}${compact ? ' compact' : ''}${s.owners.length > 1 && s.own === 'theirs' ? ' shared' : ''}`} style={empireVars(tint)} data-source={s.id}>
    <header className="vh-band">
      <span className="vh-band-kind">
        {bandOwner ? <span className="vh-band-crest"><CrestMedallion faction={bandOwner.faction} color={bandOwner.ink} size={15} /></span> : s.own === 'open' ? <span className="vh-band-open" /> : <Star size={8} />}
        <span className="vh-band-text">{ownWords(model, s)}</span>
      </span>
    </header>
    <VpSeal vp={s.vp} />
    <div className="vh-card-head">
      <SourceMedal model={model} s={s} size={compact ? 26 : 30} />
      <h3>{s.name}</h3>
    </div>
    <p className="vh-rule">{s.rule}</p>
    {terms && <p className="vh-terms">{s.terms}</p>}
    {s.kind === 'era' && <EraObjectives model={model} s={s} />}
    {s.kind === 'era' && s.era && <p className="vh-private">{s.era.myScored ? 'You have scored this Age.' : 'Every empire sees this card the turn the first empire enters the era, and it never expires.'}</p>}
    {(s.kind === 'work' || s.kind === 'era') && <ControlRow model={model} s={s} size={sockets} />}
    {s.kind === 'work' && <WorkLines model={model} s={s} />}
    {s.kind === 'goal' && s.progress && <Meter p={s.progress} color={goalOwner?.ink} />}
    {s.kind === 'goal' && s.live && !s.held && s.faction && liveWords(s.faction) && <p className="vh-private">{s.own === 'mine' ? 'Held now. ' : ''}{liveWords(s.faction)}</p>}
    {s.kind === 'goal' && s.live && !s.held && s.faction && !liveWords(s.faction) && s.own === 'mine' && <p className="vh-private">Held now, and lost the turn it falls below {(s.progress?.target ?? 0).toLocaleString('en-US')}</p>}
    {s.kind === 'goal' && s.held && <HeldLines model={model} s={s} goalOwner={goalOwner} />}
    {s.kind === 'goal' && s.privacy && s.own === 'open' && <p className="vh-private">{s.facts[0] ? `${s.facts[0].label}. ` : ''}{s.privacy}</p>}
    {s.own === 'mine' && <span className="vh-corner" aria-hidden="true"><Star size={9} /></span>}
    {children}
  </As>;
}
