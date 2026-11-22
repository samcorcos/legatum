// Thin levies (the Byzantium): armed units fielded against the cap, wherever the army is counted.
import { THIN_LEVIES } from '../data';
import { rosterLine } from '../company';
import { useGame } from '../store';
import type { LevyView, PlayerView } from '../types';
import { Icon } from './bits';
import { Amounts } from './Amounts';
import { Tip } from './Tip';
import './Levy.css';

export const LEVY_CHIP_LABEL = 'Military units';
const levyText = (l: LevyView) => `${LEVY_CHIP_LABEL}: ${l.units} of ${l.cap}`;
const levyFull = (l: LevyView) => l.units >= l.cap;
/** The viewer's own levy, when their faction has one. */
const ownLevy = (view: PlayerView | null | undefined): LevyView | undefined => view?.players.find((p) => p.id === view.me)?.levy;
const useOwnLevy = () => useGame((s) => ownLevy(s.views?.[s.seat]));

/** An empire's hired Companies, next to its levy line: how many, what they cost and when the soonest goes home. Public, like the countdowns on the units. */
function CompanyRoster({ view, owner, className = 'levy-note' }: { view: PlayerView; owner: number; className?: string }) {
  const count = view.players.find((p) => p.id === owner)?.companies ?? 0;
  if (!count) return null;
  return <p className={className} data-testid="company-roster"><Amounts text={rosterLine(view, owner, count)} /></p>;
}

/** The top-bar figure: compact, and a warning once the cap is reached (no new armed unit can start, complete or be bought). */
export function LevyCounter() {
  const l = useOwnLevy();
  const view = useGame((s) => s.views?.[s.seat]);
  if (!l) return null;
  const full = levyFull(l);
  return <Tip className={`levy-counter${full ? ' full' : ''}`} content={<>
    <div><b>{levyText(l)}</b></div>
    <div>{THIN_LEVIES}</div>
    {view && <CompanyRoster view={view} owner={view.me} className="levy-tip-company" />}
    {l.units > l.cap && <div>Over the limit by {l.units - l.cap}: no unit is disbanded, but none can be raised until you are back under it.</div>}
    {full && <div className="levy-tip-full">Levies full: no new military unit can be started, bought or finished until you have one unit fewer or one city more.</div>}
  </>}>
    <span aria-label={levyText(l)}>{full && <Icon name="ui-alert" size={12} className="ui-mark" />} <Icon name="unit-soldier" size={12} className="count-icon ui-mark" label={LEVY_CHIP_LABEL} /><span className="count-label">Units </span><b>{l.units}/{l.cap}</b>{full && ' · full'}</span>
  </Tip>;
}

/** Under the city panel's Units heading: the same figure, and the reason every armed unit below is greyed once it is full. */
export function LevyNote() {
  const l = useOwnLevy();
  const view = useGame((s) => s.views?.[s.seat]);
  if (!l) return null;
  const full = levyFull(l);
  return <>
    <p className={`levy-note${full ? ' full' : ''}`} data-testid="levy-note">
      {full && <Icon name="ui-alert" size={12} className="ui-mark" />} {levyText(l)}{full ? ': levies full, so the military units below cannot be started or bought' : ' (one per city, plus one; aircraft count, Scouts do not)'}
    </p>
    {view && <CompanyRoster view={view} owner={view.me} />}
  </>;
}

/** A rival's figure, as the public view shows it: "Thin levies: 3 of 4 armed units (full)". */
const rivalLevyText = (l: LevyView) => `Thin levies: ${l.units} of ${l.cap} military units${levyFull(l) ? ' (full: it cannot raise another)' : ''}`;

/** On a rival's diplomacy row: the cap shows how much army it can field, and when it is stuck. */
export function RivalLevy({ view, team }: { view: PlayerView; team: number }) {
  const levies = view.players.filter((p) => p.team === team && p.alive && p.levy);
  if (!levies.length) return null;
  return <>{levies.map((p) => <span key={p.id}><p className="levy-line" data-testid="rival-levy">{rivalLevyText(p.levy!)}</p><CompanyRoster view={view} owner={p.id} className="levy-line" /></span>)}</>;
}
