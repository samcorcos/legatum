// Era objectives, everywhere you look (legatum-ui "era objectives are always visible with progress"): this is how you win.
// `ObjectiveRows` is the one list of an era's three objectives with your progress; the race panel's block, the era-up
// takeover and the research screen all draw it, so a rule reads and counts the same on every surface.
import { useGame } from '../store';
import { eraName } from './eraResearch';
import { ERA_COUNT } from '../data';
import { Icon } from './bits';
import { Tip } from './Tip';
import { CrestMedallion } from '../brand/ornaments';
import { cardHint, cardName, scoredCard } from '../ageCards';
import { eraHud, eraObjectivesFor, FAMILY_LABEL, glance, howYouWin, type EraHud, type EraObjectives, type HowYouWin, type ObjectiveLine } from '../eraObjectives';
import type { PlayerView } from '../types';
import type { VpModel } from '../vpModel';
import { noEraHud, setEraHudOpen, useEraHudOpen } from './eraHudOpen';
import './eraObjectives.css';

const PART_ICON = { eras: 'milestone-six-techs', work: 'milestone-relay-uplink', goal: 'vp' } as const;
/** The one rule behind every count (R1), shown wherever progress is. */
const COUNT_NOTE = 'Counts only what you gain after you enter the era and after your last era score';
export const afterTechnologies = (left: number) => `after ${left} more ${left === 1 ? 'technology' : 'technologies'}`;
const open = () => useGame.getState().set({ vpExpanded: true });
const COUNT_UNIT: Record<string, string> = {
  Cities: 'cities', Population: 'population', Territory: 'tiles', Buildings: 'cities with this building',
  Techs: 'technologies', Captures: 'captures', CapturesFromAhead: 'captures from empires ahead of you',
  CapturesFromLeader: 'captures from the leader', Kills: 'kills',
};

/** "2/6" with a bar. Every objective counts only what you gain after entering the era and after your last era score, so an era you have not entered shows nothing yet. */
function Count({ l, preview }: { l: ObjectiveLine; preview: boolean }) {
  const note = COUNT_NOTE;
  if (l.scoredHere) return <span className="eo-num scored"><Icon name="ui-check" size={11} className="ui-mark" /> Scored</span>;
  if (preview) return <span className="eo-num" title={note}><b>0</b>/{l.target}</span>;
  if (l.met && !preview) return <span className="eo-num met" title={note}><Icon name="ui-check" size={11} className="ui-mark" /> Met</span>;
  return <span className="eo-num" title={note}><b>{l.shown}</b>/{l.target}</span>;
}

const FAMILY_ICON: Record<ObjectiveLine['family'], string> = { Build: 'building-workshop', Expand: 'city', Fight: 'unit-soldier' };

/**
 * The three objectives of one era. `preview` is an era you have not entered: its numbers are what you hold now.
 * `compact` (the race panel) shows the family as an icon, and the progress as the fill behind the row; elsewhere the family is a word and the bar its own line.
 */
export function ObjectiveRows({ lines, preview = false, big = false, inline = false, compact = false, accounting = false, view, era }: { lines: ObjectiveLine[]; preview?: boolean; big?: boolean; inline?: boolean; compact?: boolean; accounting?: boolean; view?: PlayerView; era?: number }) {
  // The line a squeezed race panel keeps (data-level 3): the one nearest done, else the first.
  const nearest = lines.reduce((best, l) => (l.shown / Math.max(1, l.target) > best.shown / Math.max(1, best.target) ? l : best), lines[0]);
  const restart = accounting && lines.find(l => !l.scoredHere && l.met && l.restartEras.length > 0);
  // An era scored with one card has nothing left to count on the others.
  const done = lines.some((l) => l.scoredHere);
  const deck = !!view?.rule_experiments?.objectives_deck;
  const turn = done && view && era ? scoredCard(view, era)?.turn : undefined;
  return <><ul className={`eo-lines${big ? ' big' : ''}${inline ? ' inline' : ''}${compact ? ' compact' : ''}${preview ? ' preview' : ''}`}>
    {lines.map((l) => {
      const family = FAMILY_LABEL[l.family];
      const idle = done && !l.scoredHere;
      const label = `${family}: ${l.text}, ${l.scoredHere ? 'scored' : idle ? 'not needed: the era is scored' : preview ? 'revealed when you enter the era' : `${l.shown} of ${l.target}`}`;
      return <li key={l.slot} className={`eo-line fam-${l.family.toLowerCase()}${l.met && !preview && !idle ? ' met' : ''}${l.scoredHere ? ' scored' : ''}${idle ? ' idle' : ''}${l === nearest ? ' nearest' : ''}`} data-testid="era-objective" data-slot={l.slot} data-family={l.family}>
        <span className="eo-fam" title={compact ? family : undefined}>
          {compact ? <><Icon name={FAMILY_ICON[l.family]} size={14} /><span className="sr-only">{family}</span></> : family}
        </span>
        <span className="eo-text" title={cardName(l.text) !== l.text ? l.text : undefined}>{cardName(l.text)}</span>
        {!idle && <Count l={l} preview={preview} />}
        {accounting && !preview && !l.scoredHere && !idle && l.countBaseline != null && l.countTotal != null && l.countingSinceTurn != null && !l.objective?.kind.match(/^(Breach|HumbleTheMighty|Veterans)$/) &&
          <span className="eo-account" data-testid="era-count-account">{l.countTotal} {COUNT_UNIT[l.kind] ?? 'total'} now · counting since turn {l.countingSinceTurn}, from {l.countBaseline}</span>}
        {deck && !compact && !inline && !preview && !l.scoredHere && !idle && cardHint(l.kind) && <span className="eo-scored-by">{cardHint(l.kind)}</span>}
        {l.scoredHere && turn !== undefined && <span className="eo-scored-by">Scored on turn {turn} with {deck ? cardName(l.text) : 'this objective'}</span>}
        {view && l.scoredBy.length > 0 && !(l.scoredHere && turn !== undefined) && <span className="eo-scored-by">Scored by {l.scoredBy.map(id => id === view.me ? 'you' : view.players.find(p => p.id === id)?.name ?? `Empire ${id}`).join(', ')}</span>}
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
  const action = open ? 'Show one line' : 'Show every era objective';
  return <>
    <header className="eo-win">
      <h4>How you win</h4>
      {scored && <span className="eo-scored" data-testid="era-scored-tag" title={`${scored}: 1 VP scored, kept for good`}>
        <Icon name="ui-check" size={11} className="ui-mark" /><span className="eo-scored-w">Era scored</span><span className="sr-only">: {scored}, 1 VP scored, kept for good</span>
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

/** The era's heading for the three objectives, from its state: a title and the short rule beside it. */
function heading(era: EraObjectives, members: number, deck = false): { title: string; rule: string } {
  const vp = members > 1 ? '1 VP each' : '1 VP';
  if (deck) return era.state === 'scored' ? { title: `${era.name} Age: 1 VP scored`, rule: 'kept for good' } : { title: `Score ${vp} in the ${era.name} Age`, rule: 'never expires' };
  if (era.state === 'preview') return { title: `Next era: ${vp} in ${era.name}`, rule: 'any one of three' };
  if (era.state === 'scored') return { title: `${era.name}: 1 VP scored`, rule: 'kept for good' };
  return { title: `Score ${vp} in ${era.name}`, rule: 'any one of three' };
}

/** A line under a preview: the first era has none, and how far you are from entering the next. */
function EntryNote({ hud }: { hud: EraHud }) {
  const e = hud.entry;
  if (!e) return null;
  const left = e.total - e.known;
  return <p className="eo-note" data-testid="era-entry-note">
    {hud.deck ? (hud.noObjective
      ? <>No Age has begun. The first begins when any empire enters the {hud.focus.name} era, and its card is shown to everyone.</>
      : hud.focus.era < ERA_COUNT ? <>The next Age begins when any empire enters the {eraName(hud.focus.era)} era. Its cards stay hidden until then.</> : null) : null}
    {!hud.deck && hud.noObjective && <>No objective in the {hud.myEraName} era. </>}
    {!hud.deck && (left > 0
      ? <>Finish {hud.noObjective ? 'its' : `the ${e.eraName} era’s`} {e.total} technologies ({e.known} known) to open these.</>
      : <>Every {e.eraName} technology is known.</>)}
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
  const label = `${tag}${hud.deck ? '' : ', any one of three objectives scores it. Closest'}: ${FAMILY_LABEL[line.family]}, ${line.text}, ${preview ? 'counts what you gain after you enter the era' : `${line.shown} of ${line.target}`}. Show every era objective`;
  return <button type="button" className={`eo-glance${line.met && !preview ? ' met' : ''}`} data-testid="era-glance" aria-expanded={false} aria-label={label} onClick={() => setEraHudOpen(true)}>
    <span className="eo-fill" aria-hidden="true"><i style={{ width: `${pct}%` }} /></span>
    <span className="eo-g-tag" title={hud.deck ? "This Age's card" : "Any one of the era's three objectives scores it; this is the closest"}>{tag}</span>
    <span className="eo-fam" title={FAMILY_LABEL[line.family]}><Icon name={FAMILY_ICON[line.family]} size={14} /></span>
    <span className="eo-text" data-testid="era-glance-text" title={cardName(line.text) !== line.text ? line.text : undefined}>{cardName(line.text)}</span>
    <Count l={line} preview={preview} />
  </button>;
}

/**
 * The race panel's block. Compact by default (legatum-ui "race rows keep their room"): the "How you win" band with your VP and one
 * line, the closest objective of the era in focus (an era already scored is the band alone, naming it). Click either to open the full block: the VP sources, this era's three objectives
 * with live progress (or a preview) and the notes.
 */
export function EraObjectivesHud({ view, model }: { view: PlayerView; model: VpModel }) {
  const hud = eraHud(view);
  const win = howYouWin(model);
  const expanded = useEraHudOpen();
  if (!hud || !win || noEraHud()) return null;
  const { focus, next } = hud;
  const h = heading(focus, win.members, hud.deck);
  const n = next ? heading(next, win.members) : null;
  const left = hud.entry ? hud.entry.total - hud.entry.known : 0;
  // An era already scored has nothing left to do this era: compact, the block is the band alone, and the band says which era is kept.
  const scoredBand = !expanded && focus.state === 'scored';
  return <section className={`era-hud st-${focus.state}${expanded ? ' expanded' : ' compact'}${scoredBand ? ' band-only' : ''}`} data-testid="era-hud" data-state={focus.state} data-era={focus.era} data-open={expanded} aria-label="How you win: era objectives">
    <HowYouWinLine win={win} open={expanded} toggle scored={scoredBand ? focus.name : undefined} />
    {!expanded && !scoredBand && <Glance hud={hud} members={win.members} />}
    {expanded && <>
      <div className="eo-era" data-testid="era-hud-current">
        <header className="eo-head">
          <h5>{focus.state === 'scored' && <Icon name="ui-check" size={12} className="ui-mark" />}{h.title}</h5>
          {focus.state === 'scored' && hud.older.length > 0
            ? <button type="button" className="link eo-rule-link" data-testid="era-older" onClick={open}>{hud.older.map((o) => o.name).join(' and ')} still open</button>
            : <><span className="eo-rule">{h.rule}</span>{hud.noObjective && !hud.deck && <span className="eo-rule short">no objective in {hud.myEraName}</span>}</>}
        </header>
        {(hud.deck || focus.state !== 'scored') && <ObjectiveRows lines={focus.lines} preview={focus.state === 'preview'} compact accounting view={view} era={focus.era} />}
      </div>
      {next && n && <div className="eo-era eo-next" data-testid="era-hud-next">
        <header className="eo-head"><h5>{n.title}</h5><span className="eo-rule">{left > 0 ? afterTechnologies(left) : n.rule}</span></header>
        <ObjectiveRows lines={next.lines} preview compact />
      </div>}
      {hud.entry && focus.state !== 'scored' && <EntryNote hud={hud} />}
      {hud.older.length > 0 && focus.state !== 'scored' && <p className="eo-note older" data-testid="era-older">
        <button type="button" className="link" onClick={open}>{hud.older.map((o) => o.name).join(' and ')} {hud.older.length > 1 ? 'are' : 'is'} still open to score</button>
      </p>}
      {hud.deck && hud.older.map(o => {
        const prior = eraObjectivesFor(view, o.era);
        return prior && <div className="eo-era eo-older" key={o.era}><header className="eo-head"><h5>{o.name} still open</h5></header><ObjectiveRows lines={prior.lines} view={view} era={o.era} compact /></div>;
      })}
    </>}
  </section>;
}

/**
 * The research screen's era header: the objectives of the era you are in with progress, and the next era's as a preview, side by
 * side. The first era has none: it says so, and shows Classical as the preview.
 */
export function EraObjectivesResearch({ view }: { view: PlayerView }) {
  const hud = eraHud(view);
  if (!hud) return null;
  const { focus } = hud;
  const r = view.research;
  // In the first era the focus is already the preview; later the next era is previewed beside the current one.
  const upcoming = hud.noObjective ? focus : hud.next ?? eraObjectivesFor(view, focus.era + 1);
  const left = r.era_total - r.era_known;
  return <section className={`era-objectives-research${hud.noObjective ? ' no-obj' : ''}`} data-testid="research-objectives" aria-label="Era objectives">
    <div className="eor-col current" data-testid="research-objectives-current">
      <h4>{hud.noObjective ? <>{hud.myEraName} era: no objective</> : focus.state === 'scored' ? <>{focus.name} {hud.deck ? 'Age' : 'era'}: 1 VP scored</> : <>Score 1 VP this {hud.deck ? 'Age' : 'era'} <small>{focus.name}{hud.deck ? '' : ' · any one of three'}</small></>}</h4>
      {hud.noObjective
        ? <p className="eo-sub"><span className="eo-by-design">The first era has none, by design. </span>Finish its {r.era_total} technologies ({r.era_known} known) to enter the {focus.name} era and open its objectives.</p>
        : focus.state === 'scored' && !hud.deck ? <p className="eo-sub">Kept for good. The next era’s objectives open when you enter it.</p> : <ObjectiveRows lines={focus.lines} inline accounting view={view} era={focus.era} />}
    </div>
    {upcoming && <div className="eor-col next" data-testid="research-objectives-next">
      <h4>Next: {upcoming.name} era <small>{left <= 0 ? 'opens when you enter it' : `opens ${afterTechnologies(left)}`}</small></h4>
      {hud.deck ? <p className="eo-sub">The {upcoming.name} Age begins when any empire enters the {upcoming.name} era.</p> : <ObjectiveRows lines={upcoming.lines} preview inline />}
    </div>}
    {hud.deck && !upcoming && focus.era < ERA_COUNT && <div className="eor-col next" data-testid="research-objectives-next"><h4>Next: {eraName(focus.era)} Age</h4><p className="eo-sub">Begins when any empire enters the {eraName(focus.era)} era.</p></div>}
  </section>;
}

/** One objective in the shared sidebar; the ledger owns the full explanation and alternatives. */
export function SidebarObjective({ view, model }: { view: PlayerView; model: VpModel }) {
  const hud = eraHud(view), win = howYouWin(model);
  if (!hud || !win) return null;
  // World Ages: the line follows the oldest open Age, which scores first.
  const focus = hud.deck && hud.oldest ? hud.oldest : hud.focus.state === 'scored' ? hud.next ?? (hud.older[0] ? eraObjectivesFor(view, hud.older[0].era) : null) : hud.focus;
  const g = focus ? glance({ ...hud, focus }, win.members) : null;
  return <section className="sidebar-objective" data-testid="era-hud" aria-label="How you win: next objective">
    <button className={`link sidebar-section-link${hud.deck ? ' age' : ''}`} onClick={open}>How you win {g?.line && <span className="sidebar-objective-stake">{g.tag}</span>}<Icon name="ui-chevron-right" size={12} /></button>
    {g?.line && focus ? <div className="sidebar-next" role="group" aria-label={`${g.tag}: ${g.line.text}`} onClick={(e) => { if (!(e.target as Element).closest('button')) open(); }}><ObjectiveRows lines={[g.line]} preview={focus.state === 'preview'} compact view={hud.deck ? view : undefined} era={focus.era} /></div> : <button className="link" onClick={open}>{hud.deck ? 'Age scored · see other ways to score' : 'Era objectives scored · see other points'}</button>}
    {hud.deck && hud.focus.era < ERA_COUNT && !hud.noObjective && <small className="eo-next-reveal">The next Age begins when any empire enters the {eraName(hud.focus.era)} era.</small>}
  </section>;
}
