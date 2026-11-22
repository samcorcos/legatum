// The body of the victory-hold alarm: what the countdown means from the viewer's side, and the
// countdown itself as a row of pips (one per turn to hold). The rule is stated, never a route:
// the user rejected suggested ways to score or to stop a rival (legatum-ui, "no suggestions").
import { empireTitle, empireMembers } from '../empires';
import { playerColor } from '../theme';
import type { PlayerView } from '../types';
import { Amounts } from '../ui/Amounts';
import { RivalWin } from '../ui/RivalWin';
import { rivalWin } from '../rivalWin';
import { Icon, ToneIcon } from '../ui/bits';
import { Ribbon, StarDivider } from '../ui/premium';
import { RingMedallion } from '../ui/ringMedallion';
import { HOLD_TURNS, turnsWord } from '../victoryHold';
import { NEAR_RULE } from '../format';
import { holdSubline, type Alarm } from './alarms';

type Tone = 'good' | 'bad' | 'neutral';

/** What the viewer is told, in the order it matters. Exported for tests. */
export function holdFacts(a: Alarm): { tone: Tone; text: string; strong?: boolean }[] {
  const h = a.hold!;
  const who = h.stance === 'self' ? 'You' : 'They';
  const facts: { tone: Tone; text: string; strong?: boolean }[] = [];
  if (h.held === 0) facts.push({ tone: 'neutral', strong: true, text: `This hold has ended: ${h.stance === 'self' ? 'your' : 'their'} VP fell below ${h.target}.` });
  else facts.push({ tone: h.stance === 'rival' ? 'bad' : 'good', strong: true, text: `${who} win the game when the count reaches ${turnsWord(h.needed)}.` });
  facts.push({ tone: 'neutral', text: `Each turn end at or above ${h.target} VP adds one turn to the count. The turn the target was reached is the first.` });
  facts.push({ tone: 'neutral', text: `Below ${h.target} VP at any turn end, the count goes back to zero.` });
  return facts;
}

function Mark({ tone }: { tone: Tone }) {
  return <span className="alarm-mark" aria-label={tone === 'bad' ? 'Bad for you' : tone === 'good' ? 'Good for you' : undefined} role={tone === 'neutral' ? undefined : 'img'}>
    <ToneIcon tone={tone} size={11} />
  </span>;
}

/** The countdown: one pip per turn to hold, filled for each held. */
function HoldMeter({ held, needed, vp, target }: { held: number; needed: number; vp: number; target: number }) {
  const ended = held === 0;
  return <div className={`hold-meter${ended ? ' ended' : ''}`}>
    <div className="hold-vp">
      <RingMedallion icon="vp" tone={ended ? 'hollow' : 'gilt'} size={40} />
      <span className="hold-vp-text"><b><Amounts text={`${vp} VP`} /></b><small>target {target}</small></span>
    </div>
    <div className="hold-count" data-testid="hold-count">
      <b>{held}</b><span> of {needed}</span>
      <small>{ended ? 'turns held' : `turns held · ${needed - held} to go`}</small>
    </div>
    <div className="hold-pips" role="meter" aria-label="Turns held" aria-valuemin={0} aria-valuemax={needed} aria-valuenow={held}
      aria-valuetext={`${held} of ${needed} turns held`} style={{ ['--n' as string]: needed }}>
      {Array.from({ length: needed }, (_, i) => <i key={i} className={i < held ? 'on' : ''} />)}
    </div>
  </div>;
}

export function HoldDetail({ alarm: a, view, named }: { alarm: Alarm; view: PlayerView; named: boolean }) {
  const h = a.hold!;
  return <section className="alarm-block hold-block" aria-label={named ? empireTitle(view, a.team) : undefined}>
    {named && <>
      <h2 className="alarm-block-head" style={{ ['--pc' as string]: playerColor(empireMembers(view, a.team)[0]?.id) }}>{empireTitle(view, a.team)}</h2>
      <p className="alarm-hold-sub small">{holdSubline(a)}</p>
    </>}
    <div className="alarm-cols hold-cols">
      <div>
        <h3>What it means now</h3>
        <ul className="alarm-facts">{holdFacts(a).map((f, i) => <li key={i} className={`alarm-fact ${f.tone}${f.strong ? ' strong' : ''}`}>
          <Mark tone={f.tone} /><span><Amounts text={f.text} /></span></li>)}</ul>
      </div>
      <div className="hold-side">
        <Ribbon tone={h.stance === 'rival' ? 'red' : h.stance === 'self' ? 'gilt' : 'ink'} className="hold-ribbon">
          <Icon name="vp" size={14} /> Hold for {turnsWord(h.needed)}
        </Ribbon>
        <HoldMeter held={h.held} needed={h.needed} vp={h.vp} target={h.target} />
      </div>
    </div>
    {h.stance === 'rival' && <RivalAlarmActions alarm={a} view={view} />}
    <StarDivider className="hold-sd" />
  </section>;
}

/** What the near-victory card says: what scored this turn, where the team stands, and what stops it, from rules that exist. */
export function nearFacts(a: Alarm, view: PlayerView): { tone: Tone; text: string; strong?: boolean }[] {
  const n = a.near!;
  const members = new Set(empireMembers(view, a.team).map((p) => p.id));
  const facts: { tone: Tone; text: string; strong?: boolean }[] = [];
  const own = n.stance === 'self';
  facts.push({ tone: n.stance === 'rival' ? 'bad' : 'good', strong: true, text: own ? `Reach ${n.target} VP and hold it for ${turnsWord(HOLD_TURNS)} to win.` : `At ${n.target} VP they must hold it for ${turnsWord(HOLD_TURNS)} to win.` });
  if (n.scored.length) for (const l of n.scored.slice(0, 3)) facts.push({ tone: 'neutral', text: l.text });
  else {
    const kinds = new Map<string, number>();
    for (const p of view.players.filter((p) => members.has(p.id))) for (const [src, vp] of p.vp_sources) kinds.set(src.kind, (kinds.get(src.kind) ?? 0) + vp);
    const names: Record<string, string> = { GreatWork: 'world wonders', FactionGoal: 'faction goals', EraObjective: 'Age cards', CapitalTaken: 'held rival capitals' };
    const text = [...kinds].filter(([, vp]) => vp > 0).map(([k, vp]) => `${vp} from ${names[k] ?? k}`).join(', ');
    if (text) facts.push({ tone: 'neutral', text: `Their VP: ${text}.` });
  }
  facts.push({ tone: 'neutral', text: NEAR_RULE });
  return facts;
}

/** The plain facts and actions against a rival close to winning; its Declare war is the card's primary, up in the actions row. */
function RivalAlarmActions({ alarm: a, view }: { alarm: Alarm; view: PlayerView }) {
  const rival = rivalWin(view, a.team);
  return rival ? <RivalWin view={view} rival={rival} declare={false} /> : null;
}

export function NearDetail({ alarm: a, view }: { alarm: Alarm; view: PlayerView }) {
  return <section className="alarm-block hold-block">
    {a.near!.stance === 'rival' && <RivalAlarmActions alarm={a} view={view} />}
    <div className="alarm-cols single">
      <div>
        <h3>What it means now</h3>
        <ul className="alarm-facts">{nearFacts(a, view).map((f, i) => <li key={i} className={`alarm-fact ${f.tone}${f.strong ? ' strong' : ''}`}>
          <Mark tone={f.tone} /><span><Amounts text={f.text} /></span></li>)}</ul>
      </div>
    </div>
  </section>;
}
