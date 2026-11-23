// World Ages, everywhere you look (legatum-ui "era objectives are always visible with progress"): this is how you win.
// `ObjectiveRows` is the one list of an Age's card with your progress; the race panel's block, the era-up takeover and the
// research screen all draw it, so a rule reads and counts the same on every surface.
import { eraName, eraNumeral } from './eraResearch';
import { RingMedallion } from './ringMedallion';
import { useEffect, useRef } from 'react';
import { useCombatFx } from '../feel';
import { useReducedMotion } from './useReducedMotion';
import { motionStill } from './motion';
import { ageName, ERA_COUNT } from '../data';
import { Icon } from './bits';
import { cardHint, cardName, scoredCard } from '../ageCards';
import { eraHud, FAMILY_LABEL, glance, howYouWin, type ObjectiveLine } from '../eraObjectives';
import { empireOfPlayer } from '../empires';
import type { PlayerView } from '../types';
import type { VpModel } from '../vpModel';
import './eraObjectives.css';

/** The one rule behind every count (R1), shown wherever progress is. */
const COUNT_NOTE = 'Counts only what you do after this Age begins and after you last scored an Age';
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
 * The research screen's Age header: the card of the Age in focus with progress, and when the next Age begins. Before any Age
 * begins it says when the first one does.
 */
/** The Age states and met cards this session has already shown on the research screen: each stamps in once, the first time it is seen. */
const seenStamps = new Set<string>();

export function EraObjectivesResearch({ view }: { view: PlayerView }) {
  const hud = eraHud(view);
  const { focus } = hud;
  const still = motionStill(useReducedMotion(), useCombatFx((st) => st.enabled));
  // Keyed by seat so each online player sees their own stamp.
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
  return <section className="sidebar-objective" data-testid="era-hud" aria-label="Next Age objective">
    <h3 className="sidebar-section-link age">Next Age {g?.line && <span className="sidebar-objective-stake">{g.tag}</span>}</h3>
    {g?.line && focus ? <div className="sidebar-next" role="group" aria-label={`${g.tag}: ${g.line.text}`}><ObjectiveRows lines={[g.line]} preview={focus.state === 'preview'} compact view={view} era={focus.era} /></div> : <div className="sidebar-next">Age scored</div>}
    {hud.focus.era < ERA_COUNT && !hud.noObjective && <small className="eo-next-reveal">The next Age begins when any empire enters the {eraName(hud.focus.era)} era.</small>}
  </section>;
}
