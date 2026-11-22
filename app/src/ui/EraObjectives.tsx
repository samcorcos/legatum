// World Ages, everywhere you look (legatum-ui "era objectives are always visible with progress"): this is how you win.
// `ObjectiveRows` is the one list of an Age's card with your progress; the race panel's block, the era-up takeover and the
// research screen all draw it, so a rule reads and counts the same on every surface.
import { useGame } from '../store';
import { eraName, eraNumeral } from './eraResearch';
import { RingMedallion } from './ringMedallion';
import { useEffect, useRef } from 'react';
import { useCombatFx } from '../feel';
import { useReducedMotion } from './useReducedMotion';
import { motionStill } from './motion';
import { ageName, ERA_COUNT } from '../data';
import { Icon } from './bits';
import { Tip } from './Tip';
import { CrestMedallion } from '../brand/ornaments';
import { cardHint, cardName, scoredCard } from '../ageCards';
import { eraHud, eraObjectivesFor, FAMILY_LABEL, glance, howYouWin, type EraHud, type EraObjectives, type HowYouWin, type ObjectiveLine } from '../eraObjectives';
import { empireOfPlayer } from '../empires';
import type { PlayerView } from '../types';
import type { VpModel } from '../vpModel';
import { noEraHud, setEraHudOpen, useEraHudOpen } from './eraHudOpen';
import './eraObjectives.css';

const PART_ICON = { eras: 'milestone-six-techs', work: 'milestone-great-library', goal: 'vp' } as const;
/** The one rule behind every count (R1), shown wherever progress is. */
const COUNT_NOTE = 'Counts only what you do after this Age begins and after you last scored an Age';
const open = () => useGame.getState().set({ vpExpanded: true });
const COUNT_UNIT: Record<string, string> = {
  Cities: 'cities', Population: 'population', Territory: 'tiles', Buildings: 'cities with this building',
};

/** "2/6" with a bar. Every card counts only what you do after its Age begins and after your last Age score, so an Age not begun shows nothing yet. */
function Count({ l, preview }: { l: ObjectiveLine; preview: boolean }) {
  const note = COUNT_NOTE;
  if (l.scoredHere) return <span className="eo-num scored"><Icon name="ui-check" size={11} className="ui-mark" /> Scored</span>;
  if (preview) return <span className="eo-num" title={note}><b>0</b>/{l.target}</span>;
  if (l.met && !preview) return <span className="eo-num met" title={note}><Icon name="ui-check" size={11} className="ui-mark" /> Met</span>;
  return <span className="eo-num" title={note}><b>{l.shown}</b>/{l.target}</span>;
}

const FAMILY_ICON: Record<ObjectiveLine['family'], string> = { Build: 'building-workshop', Expand: 'city', Fight: 'unit-spearman' };

/**
 * An Age's card (a list, so a test fixture can carry more than one). `preview` is an Age that has not begun: it counts nothing yet.
 * `compact` (the race panel) shows the family as an icon, and the progress as the fill behind the row; elsewhere the family is a word and the bar its own line.
 */
export function ObjectiveRows({ lines, preview = false, big = false, inline = false, compact = false, accounting = false, view, era, flash = [] }: { lines: ObjectiveLine[]; preview?: boolean; big?: boolean; inline?: boolean; compact?: boolean; accounting?: boolean; view?: PlayerView; era?: number; flash?: number[] }) {
  // The line a squeezed race panel keeps (data-level 3): the one nearest done, else the first.
  const nearest = lines.reduce((best, l) => (l.shown / Math.max(1, l.target) > best.shown / Math.max(1, best.target) ? l : best), lines[0]);
  const restart = accounting && lines.find(l => !l.scoredHere && l.met && l.restartEras.length > 0);
  // An era scored with one card has nothing left to count on the others.
  const done = lines.some((l) => l.scoredHere);
  const turn = done && view && era ? scoredCard(view, era)?.turn : undefined;
  return <><ul className={`eo-lines${big ? ' big' : ''}${inline ? ' inline' : ''}${compact ? ' compact' : ''}${preview ? ' preview' : ''}`}>
    {lines.map((l) => {
      const family = FAMILY_LABEL[l.family];
      const idle = done && !l.scoredHere;
      const label = `${family}: ${l.text}, ${l.scoredHere ? 'scored' : idle ? `not needed: the Age is scored` : preview ? 'revealed when the first empire enters the era' : `${l.shown} of ${l.target}`}`;
      return <li key={l.slot} className={`eo-line fam-${l.family.toLowerCase()}${l.met && !preview && !idle ? ' met' : ''}${l.scoredHere ? ' scored' : ''}${idle ? ' idle' : ''}${l === nearest ? ' nearest' : ''}${flash.includes(l.slot) ? ' flash' : ''}`} data-testid="era-objective" data-slot={l.slot} data-family={l.family}>
        <span className="eo-fam" title={compact || inline ? family : undefined}>
          {compact || inline ? <><Icon name={FAMILY_ICON[l.family]} size={14} /><span className="sr-only">{family}</span></> : family}
        </span>
        {inline && l.text.includes(':')
          ? <span className="eo-text" title={l.text}>{cardName(l.text)} <span className="eo-ask">· {l.text.slice(l.text.indexOf(':') + 1).trim()}</span></span>
          : <span className="eo-text" title={cardName(l.text) !== l.text ? l.text : undefined}>{cardName(l.text)}</span>}
        {!idle && <Count l={l} preview={preview} />}
        {accounting && !preview && !l.scoredHere && !idle && l.countBaseline != null && l.countTotal != null && l.countingSinceTurn != null && !l.objective?.kind.match(/^(Breach|HumbleTheMighty|Veterans)$/) &&
          <span className="eo-account" data-testid="era-count-account">{l.countTotal} {COUNT_UNIT[l.kind] ?? 'total'} now · counting since turn {l.countingSinceTurn}, from {l.countBaseline}</span>}
        {!compact && !inline && !preview && !l.scoredHere && !idle && cardHint(l.kind) && <span className="eo-scored-by">{cardHint(l.kind)}</span>}
        {l.scoredHere && turn !== undefined && <span className="eo-scored-by">Scored on turn {turn} with {cardName(l.text)}</span>}
        {view && l.scoredBy.length > 0 && !(l.scoredHere && turn !== undefined) && <span className="eo-scored-by">Scored by {l.scoredBy.map(id => id === view.me ? 'you' : empireOfPlayer(view, id)).join(', ')}</span>}
        <span className="eo-bar" role="progressbar" aria-label={label} aria-valuemin={0} aria-valuemax={l.target} aria-valuenow={preview || idle ? 0 : l.shown}>
          <i style={{ width: `${preview || idle ? 0 : l.target > 0 ? Math.min(100, (l.shown / l.target) * 100) : 0}%` }} />
        </span>
      </li>;
    })}
  </ul>{restart && view && <p className="eo-restart" data-testid="era-count-restart">When this scores, your {restart.restartEras.map(e => view.eras.find(v => v.era === e)?.name ?? `era ${e}`).join(' and ')} progress goes back to zero.</p>}</>;
}

/**
 * "How you win": the band with your VP towards the target, which expands or compacts the block (a button), and, when expanded,
 * the VP sources behind it with your count in each.
 */
function HowYouWinLine({ win, open, toggle, scored }: { win: HowYouWin; open: boolean; toggle?: boolean; scored?: string }) {
  const action = open ? 'Show one line' : 'Show every Age card';
  return <>
    <header className="eo-win">
      <h4>How you win</h4>
      {scored && <span className="eo-scored" data-testid="era-scored-tag" title={`${scored}: 1 VP scored, kept for good`}>
        <Icon name="ui-check" size={11} className="ui-mark" /><span className="eo-scored-w">Age scored</span><span className="sr-only">: {scored}, 1 VP scored, kept for good</span>
      </span>}
      <span className="eo-vp" data-testid="how-you-win-vp"><b>{win.vp}</b> of {win.target} VP</span>
      {toggle && <button type="button" className="eo-toggle" data-testid="era-hud-toggle" aria-expanded={open} onClick={() => setEraHudOpen(!open)}
        title={scored ? `${scored}: 1 VP scored, kept for good. ${action}` : action}>
        <Icon name={open ? 'ui-chevron-up' : 'ui-chevron-down'} size={12} className="ui-mark" />
        <span className="sr-only">{action}</span>
      </button>}
    </header>
    {open && <ul className="eo-parts" aria-label={`Where your ${win.target} VP come from`}>
      {win.parts.map((p) => <li key={p.key} className={`eo-part ${p.key}${p.have > 0 ? ' has' : ''}`} data-part={p.key}>
        <Tip content={p.title}><span>
          {p.key === 'goal' ? <CrestMedallion faction={win.faction} size={14} color={win.ink} /> : <Icon name={PART_ICON[p.key]} size={14} />}
          <span className="sr-only">{p.label}</span> <b>{p.max === null ? p.have : `${p.have}/${p.max}`}</b>
        </span></Tip>
      </li>)}
    </ul>}
  </>;
}

/** The Age's heading for its card, from its state: a title and the short rule beside it. */
function heading(era: EraObjectives, members: number): { title: string; rule: string } {
  const vp = members > 1 ? '1 VP each' : '1 VP';
  return era.state === 'scored' ? { title: `${ageName(era.name)}: 1 VP scored`, rule: 'kept for good' } : { title: `Score ${vp} in the ${ageName(era.name)}`, rule: 'never expires' };
}

/** A line under the card: when the first or next Age begins. */
function EntryNote({ hud }: { hud: EraHud }) {
  if (!hud.entry) return null;
  return <p className="eo-note" data-testid="era-entry-note">
    {hud.noObjective
      ? <>No Age has begun. The first begins when any empire enters the {hud.focus.name} era, and its card is shown to everyone.</>
      : hud.focus.era < ERA_COUNT ? <>The next Age begins when any empire enters the {eraName(hud.focus.era)} era. Its card stays hidden until then.</> : null}
  </p>;
}

/** The compact block's one line: the era in focus and its objective nearest done, with its progress as the fill behind the line. A click opens the full block. */
function Glance({ hud, members }: { hud: EraHud; members: number }) {
  const g = glance(hud, members);
  const tag = g.tag;
  const line = g.line;
  if (!line) return null;
  const preview = hud.focus.state === 'preview';
  const pct = preview ? 0 : line.target > 0 ? Math.min(100, (line.shown / line.target) * 100) : 0;
  const label = `${tag}: ${FAMILY_LABEL[line.family]}, ${line.text}, ${preview ? 'counts what you do after the Age begins' : `${line.shown} of ${line.target}`}. Show every Age card`;
  return <button type="button" className={`eo-glance${line.met && !preview ? ' met' : ''}`} data-testid="era-glance" aria-expanded={false} aria-label={label} onClick={() => setEraHudOpen(true)}>
    <span className="eo-fill" aria-hidden="true"><i style={{ width: `${pct}%` }} /></span>
    <span className="eo-g-tag" title="This Age's card">{tag}</span>
    <span className="eo-fam" title={FAMILY_LABEL[line.family]}><Icon name={FAMILY_ICON[line.family]} size={14} /></span>
    <span className="eo-text" data-testid="era-glance-text" title={cardName(line.text) !== line.text ? line.text : undefined}>{cardName(line.text)}</span>
    <Count l={line} preview={preview} />
  </button>;
}

/**
 * The race panel's block. Compact by default (legatum-ui "race rows keep their room"): the "How you win" band with your VP and one
 * line, the card of the Age in focus (an Age already scored is the band alone, naming it). Click either to open the full block: the
 * VP sources, the Age's card with live progress, every earlier Age still open, and the notes.
 */
export function EraObjectivesHud({ view, model }: { view: PlayerView; model: VpModel }) {
  const hud = eraHud(view);
  const win = howYouWin(model);
  const expanded = useEraHudOpen();
  if (!win || noEraHud()) return null;
  const { focus } = hud;
  const h = heading(focus, win.members);
  // An Age already scored has nothing left to do: compact, the block is the band alone, and the band says which Age is kept.
  const scoredBand = !expanded && focus.state === 'scored';
  return <section className={`era-hud st-${focus.state}${expanded ? ' expanded' : ' compact'}${scoredBand ? ' band-only' : ''}`} data-testid="era-hud" data-state={focus.state} data-era={focus.era} data-open={expanded} aria-label="How you win: World Ages">
    <HowYouWinLine win={win} open={expanded} toggle scored={scoredBand ? focus.name : undefined} />
    {!expanded && !scoredBand && <Glance hud={hud} members={win.members} />}
    {expanded && <>
      <div className="eo-era" data-testid="era-hud-current">
        <header className="eo-head">
          <h5>{focus.state === 'scored' && <Icon name="ui-check" size={12} className="ui-mark" />}{h.title}</h5>
          {focus.state === 'scored' && hud.older.length > 0
            ? <button type="button" className="link eo-rule-link" data-testid="era-older" onClick={open}>{hud.older.map((o) => o.name).join(' and ')} still open</button>
            : <span className="eo-rule">{h.rule}</span>}
        </header>
        {<ObjectiveRows lines={focus.lines} preview={focus.state === 'preview'} compact accounting view={view} era={focus.era} />}
      </div>
      {hud.entry && focus.state !== 'scored' && <EntryNote hud={hud} />}
      {hud.older.length > 0 && focus.state !== 'scored' && <p className="eo-note older" data-testid="era-older">
        <button type="button" className="link" onClick={open}>{hud.older.map((o) => o.name).join(' and ')} {hud.older.length > 1 ? 'are' : 'is'} still open to score</button>
      </p>}
      {hud.older.map(o => {
        const prior = eraObjectivesFor(view, o.era);
        return prior && <div className="eo-era eo-older" key={o.era}><header className="eo-head"><h5>{o.name} still open</h5></header><ObjectiveRows lines={prior.lines} view={view} era={o.era} compact /></div>;
      })}
    </>}
  </section>;
}

/**
 * The research screen's Age header: the card of the Age in focus with progress, and when the next Age begins. Before any Age
 * begins it says when the first one does.
 */
/** The Age states and met cards this session has already shown on the research screen: each stamps in once, the first time it is seen. */
const seenStamps = new Set<string>();

export function EraObjectivesResearch({ view }: { view: PlayerView }) {
  const hud = eraHud(view);
  const { focus } = hud;
  const still = motionStill(useReducedMotion(), useCombatFx((st) => st.enabled));
  // Keyed by seat, so each hot-seat player sees their own stamp.
  const sealKey = `${view.me}:${focus.era}:${focus.state}`;
  const metSlots = focus.lines.filter((l) => l.met).map((l) => l.slot);
  const metKey = (slot: number) => `${view.me}:${focus.era}:${slot}:met`;
  // A newly opened or newly scored Age stamps its seal in; a card newly met flashes once. Never under reduced motion or with
  // animations off. Decided once per key while mounted, so a re-render mid-animation does not cut it short.
  const decided = useRef(new Map<string, boolean>());
  const decide = (key: string) => {
    if (!decided.current.has(key)) decided.current.set(key, !still && !seenStamps.has(key));
    return decided.current.get(key)!;
  };
  const stamp = !hud.noObjective && decide(sealKey);
  const flash = hud.noObjective ? [] : metSlots.filter((slot) => decide(metKey(slot)));
  useEffect(() => { if (!hud.noObjective) [sealKey, ...metSlots.map(metKey)].forEach((k) => seenStamps.add(k)); });
  return <section className={`era-objectives-research${hud.noObjective ? ' no-obj' : ''}`} data-testid="research-objectives" aria-label="World Ages">
    {hud.noObjective
      ? <div className="eor-col current" data-testid="research-objectives-current">
        <h4>No Age has begun</h4>
        <p className="eo-sub">The first Age begins when any empire enters the {focus.name} era, and its card is shown to everyone.</p>
      </div>
      // An open Age is how you win: its era seal beside the card makes it the screen's big moment, not a status line.
      : <div className={`eor-col current age st-${focus.state}`} data-testid="research-objectives-current">
        <RingMedallion icon={`era-numeral:${eraNumeral(focus.era - 1)}`} tone={focus.state === 'scored' ? 'gilt' : 'hollow'} size={46} className={`eor-seal${stamp ? ' stamp' : ''}`} />
        <div className="eor-body">
          <header className="eor-head">
            <h4>{focus.state === 'scored' ? <>{ageName(focus.name)}: 1 VP scored</> : <>{ageName(focus.name)}: score 1 VP</>}</h4>
            {focus.era < ERA_COUNT && <p className="eo-sub eor-next" data-testid="research-objectives-next">The next Age begins when any empire enters the {eraName(focus.era)} era.</p>}
          </header>
          <ObjectiveRows lines={focus.lines} inline accounting view={view} era={focus.era} flash={flash} />
        </div>
      </div>}
  </section>;
}

/** One objective in the shared sidebar; the ledger owns the full explanation and alternatives. */
export function SidebarObjective({ view, model }: { view: PlayerView; model: VpModel }) {
  const hud = eraHud(view), win = howYouWin(model);
  if (!win) return null;
  // The line follows the oldest open Age, which scores first.
  const focus = hud.oldest ?? (hud.focus.state === 'scored' ? null : hud.focus);
  const g = focus ? glance({ ...hud, focus }, win.members) : null;
  return <section className="sidebar-objective" data-testid="era-hud" aria-label="How you win: next objective">
    <button className="link sidebar-section-link age" onClick={open}>How you win {g?.line && <span className="sidebar-objective-stake">{g.tag}</span>}<Icon name="ui-chevron-right" size={12} /></button>
    {g?.line && focus ? <div className="sidebar-next" role="group" aria-label={`${g.tag}: ${g.line.text}`} onClick={(e) => { if (!(e.target as Element).closest('button')) open(); }}><ObjectiveRows lines={[g.line]} preview={focus.state === 'preview'} compact view={view} era={focus.era} /></div> : <button className="link" onClick={open}>Age scored · see other ways to score</button>}
    {hud.focus.era < ERA_COUNT && !hud.noObjective && <small className="eo-next-reveal">The next Age begins when any empire enters the {eraName(hud.focus.era)} era.</small>}
  </section>;
}
