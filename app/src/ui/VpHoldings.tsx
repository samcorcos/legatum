// The race panel's chips and hold strips: the public facts a rivals-table row carries beside its name
// (a closed country, Egypt's tomb work, lead and decadence, a fallen capital) and the countdown strips of
// a team holding its VP target. Every way to score, token by token, lives in the Score ledger.
import { Star } from '../brand/ornaments';
import { DECADENCE_FOOD_RULE, POWER_RULE, decadenceCurveText, decadenceRules, type DecadenceRules } from '../data';
import { Icon } from './bits';
import { Tip } from './Tip';
import { sakokuChip, sakokuLine } from '../sakoku';
import { tombChip, tombPublicLine, tombShort } from '../goalCities';
import { empireVars } from './vpParts';
import type { VpEmpire, VpModel, VpRow } from '../vpModel';
import './vpHoldings.css';

/** Whether a team still holds an original capital of its own: a star and the word, never colour alone. Public, like every row's VP. */
export function CapitalChip({ row, nameOf }: { row: VpRow; nameOf: (e: VpEmpire) => string }) {
  const who = row.isMine ? (row.members.length > 1 ? 'Your team' : 'You') : row.label;
  const detail = row.members.length > 1 ? ` ${row.members.map((m) => `${nameOf(m)}: ${m.capitalStands ? 'standing' : 'fallen'}`).join('; ')}.` : '';
  const title = row.capitalStands
    ? `${who}: an original capital of its own still stands.${detail}`
    : `${who}: every original capital of its own has been taken or razed, so it cannot win by holding the last capital unless it retakes one. It can still win by VP.`;
  return <Tip hoverOnly content={title}><span className={`vhc-capital${row.capitalStands ? '' : ' fallen'}`} data-testid="capital-chip" data-team={row.team} data-stands={row.capitalStands ? 'true' : 'false'}
    aria-label={`${who}: original capital ${row.capitalStands ? 'standing' : 'fallen'}`}>
    <Star size={9} />{row.capitalStands ? 'Capital' : 'Capital fallen'}
  </span></Tip>;
}

/** A legacy Sakoku scenario's public vow, with its countdown and full rule. */
export function ClosedChip({ members }: { members: VpEmpire[] }) {
  const closed = members.filter((m) => m.sakoku);
  if (!closed.length) return null;
  return <>{closed.map((m) => {
    const st = m.sakoku!;
    const title = `${m.isMe ? 'You have' : `${m.name} has`} closed the country (Sakoku): ${(() => { const l = sakokuLine(st); return l.charAt(0).toLowerCase() + l.slice(1); })()}. While closed it can't declare war or ally, learns nothing from abroad, and nobody may cross its land at Peace.`;
    return <Tip key={m.id} hoverOnly content={title}><span className={`vhc-closed${st.holding ? ' holding' : st.left === 0 ? ' blocked' : ''}`} data-testid="closed-chip" data-owner={m.id}
      aria-label={`${m.isMe ? 'Your country is' : `${m.name} is`} closed: ${sakokuLine(st)}`}>
      <Icon name="ui-sakoku" size={11} />{sakokuChip(st)}
    </span></Tip>;
  })}</>;
}

/** Egypt's public Royal Tomb work on its race row: the count toward the tomb, and whether the work is on (its cities stopped) or paused. */
export function TombChip({ members }: { members: VpEmpire[] }) {
  const working = members.filter((m) => m.tomb);
  if (!working.length) return null;
  return <>{working.map((m) => {
    const st = m.tomb!;
    const title = m.isMe
      ? `${tombShort(st)}. ${st.working ? 'None of your cities builds or grows while the work is on.' : 'Paused: your cities build and grow; the count is kept.'}`
      : `${m.name}: ${tombPublicLine(st)}`;
    return <Tip key={m.id} hoverOnly content={title}><span className={`vhc-closed vhc-tomb${st.working ? ' working' : ''}`} data-testid="tomb-chip" data-owner={m.id}
      aria-label={`${m.isMe ? 'Your' : `${m.name}'s`} ${tombShort(st)}${st.working ? '' : ', paused'}`}>
      <Icon name="goal-royal-tomb" size={11} />{tombChip(st)}
    </span></Tip>;
  })}</>;
}

/** How far ahead of the average empire a row's empire is ("+60%", neutral: the cost is the red chip beside it), with the one rate it causes in the tooltip. */
export function LeadChip({ members, rules = decadenceRules() }: { members: VpEmpire[]; rules?: DecadenceRules }) {
  if (rules.decadenceSlope === 0) return null;
  const ahead = members.filter((m) => m.lead > 0);
  if (!ahead.length) return null;
  const top = ahead.reduce((a, b) => (b.lead > a.lead ? b : a));
  const says = (m: VpEmpire) => {
    const who = m.isMe ? 'Your' : `${m.name}'s`, its = m.isMe ? 'your' : 'its';
    return `${who} Power is ${m.lead}% above the average empire's. ${m.decadence > 0
      ? `The further ahead an empire is, the more its people grow complacent, so ${its} cities grow ${m.decadence}% slower and lose ${m.decadence}% of their Production, Gold and Science.`
      : `That is too little to cut any yield yet.`}`;
  };
  const title = ahead.map(says).join(' ') + ` ${POWER_RULE} ${decadenceCurveText(rules)}`;
  return <Tip hoverOnly content={title}><span className="vhc-lead" data-testid="lead-chip" aria-label={`Power ${top.lead}% above the average empire`}>
    +{top.lead}%
  </span></Tip>;
}

/** The rate that lead costs: every yield of the row's empire cut by this percent (a red chip with a down arrow, since colour is never alone). */
export function DecadenceChip({ members, rules = decadenceRules() }: { members: VpEmpire[]; rules?: DecadenceRules }) {
  if (rules.decadenceSlope === 0) return null;
  const cut = members.filter((m) => m.decadence > 0);
  if (!cut.length) return null;
  const top = cut.reduce((a, b) => (b.decadence > a.decadence ? b : a));
  const title = cut.map((m) => `${m.isMe ? 'Your' : `${m.name}'s`} cities grow ${m.decadence}% slower (smaller families) and lose ${m.decadence}% of their Production (sloth), Gold (luxury) and Science (complacency).`).join(' ')
    + ` ${DECADENCE_FOOD_RULE}`;
  return <Tip hoverOnly content={title}><span className="vhc-decadence" data-testid="decadence-chip" aria-label={`Decadence: growth ${top.decadence}% slower, Production, Gold and Science cut by ${top.decadence}%`}>
    <Icon name="ui-trend-down" size={9} className="ui-mark" />{top.decadence}%
  </span></Tip>;
}

/** What a race row says about the relation with its empire: a pin on the roundel, the words in its tooltip and name. */
export interface RowRelation { kind: 'war' | 'peace' | 'alliance'; declared?: boolean; isNew?: boolean; text: string; title: string }

/** The countdown wording, one place: "You win in 7 turns" / "Mongols Kin wins in 1 turn" / "wins this turn". */
export function holdWords(row: VpRow, { short = false }: { short?: boolean } = {}): string {
  const left = row.hold!.left;
  const when = left === 0 ? 'this turn' : `in ${left} turn${left === 1 ? '' : 's'}`;
  if (short) return left === 0 ? 'wins now' : `wins in ${left}`;
  return `${row.isMine ? (row.members.length > 1 ? 'Your team wins' : 'You win') : `${row.label} wins`} ${when}`;
}

/** The countdown with the fact first, for a narrow strip that must never cut it: "Wins in 7 turns", "You win in 1 turn". */
function holdHeadline(row: VpRow): string {
  const left = row.hold!.left;
  const when = left === 0 ? 'this turn' : `in ${left} turn${left === 1 ? '' : 's'}`;
  return `${row.isMine ? (row.members.length > 1 ? 'Your team wins' : 'You win') : 'Wins'} ${when}`;
}

/** One pip per turn of the hold: filled for turns held, empty for turns still to hold. */
function HoldMeter({ row }: { row: VpRow }) {
  const h = row.hold!;
  return <span className="vhc-hold-meter" role="progressbar" aria-label={`${row.label}: holds the target ${h.turnsHeld} of ${h.turnsNeeded} turns`} aria-valuemin={0} aria-valuemax={h.turnsNeeded} aria-valuenow={Math.min(h.turnsHeld, h.turnsNeeded)}>
    {Array.from({ length: h.turnsNeeded }, (_, i) => <i key={i} className={i < h.turnsHeld ? 'on' : ''} />)}
  </span>;
}

/**
 * Teams holding their VP target are counting down to the win: the race says so above its rows,
 * in the empire's own colour, a red-ringed alarm when it is a rival's and gilt when it is yours.
 * Two strips at most in the race; the sheet shows them all.
 */
/** `named`: the headline carries the empire's name (a wide strip), instead of leaving it to the line under it. */
export function HoldStrips({ model, limit = 2, named = false }: { model: VpModel; limit?: number; named?: boolean }) {
  if (!model.holds.length) return null;
  const shown = model.holds.slice(0, limit);
  return <div className="vhc-holds-list" role="group" aria-label="Teams holding their victory target">
    {shown.map((row) => (
      <div key={row.team} className={`vhc-hold ${row.isMine ? 'mine' : 'rival'}${row.hold!.left <= 2 ? ' last' : ''}`} role="status" aria-label={`${holdWords(row)}: holding ${row.vp} of ${row.target} VP`} style={empireVars(row.members[0])}>
        <span className="vhc-hold-icon">{row.isMine ? <Star size={9} /> : <Icon name="ui-alert" size={12} className="ui-mark" />}</span>
        <span className="vhc-hold-text">
          <b>{named ? holdWords(row) : holdHeadline(row)}</b>
          <small>{row.isMine || named ? `Holding ${row.vp} of ${row.target} VP` : `${row.label} · holding ${row.vp} of ${row.target} VP`}</small>
        </span>
        <HoldMeter row={row} />
      </div>
    ))}
    {model.holds.length > shown.length && <div className="vhc-hold-more">+{model.holds.length - shown.length} more holding</div>}
  </div>;
}
