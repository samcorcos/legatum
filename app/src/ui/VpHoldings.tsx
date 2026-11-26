// The race panel's chips and hold strips: the public facts a rivals-table row carries beside its name
// (a closed country, Egypt's tomb work, lead and decadence, a fallen capital) and the countdown strips of
// a team holding its VP target. Every way to score, token by token, lives in the Score ledger.
import { Star } from '../brand/ornaments';
import { DECADENCE_FOOD_RULE, POWER_RULE, decadenceCurveText, decadenceRules, type DecadenceRules } from '../data';
import { Icon } from './bits';
import { sakokuChip, sakokuLine } from '../sakoku';
import { tombChip, tombPublicLine, tombShort } from '../goalCities';
import { empireVars } from './vpParts';
import type { VpEmpire, VpModel, VpRow } from '../vpModel';
import './vpHoldings.css';

/** Whether a team still holds an original capital of its own: a star and the word, never colour alone. Public, like every row's VP. */
export function CapitalChip({ row, nameOf }: { row: VpRow; nameOf: (e: VpEmpire) => string }) {
  const who = !(row.isMine) ? row.label : (!(row.members.length > 1) ? 'You' : 'Your team');
  const detail = !(row.members.length > 1) ? '' : ` ${row.members.map((m) => `${nameOf(m)}: ${!(m.capitalStands) ? 'fallen' : 'standing'}`).join('; ')}.`;
  const title = !(row.capitalStands) ? `${who}: every original capital of its own has been taken or razed, so it cannot win by holding the last capital unless it retakes one. It can still win by VP.` : `${who}: an original capital of its own still stands.${detail}`;
  return <span className={`vhc-capital${!(row.capitalStands) ? ' fallen' : ''}`} title={title} data-testid="capital-chip" data-team={row.team} data-stands={!(row.capitalStands) ? 'false' : 'true'}
    aria-label={`${who}: original capital ${!(row.capitalStands) ? 'fallen' : 'standing'}`}>
    <Star size={9} />{!(row.capitalStands) ? 'Capital fallen' : 'Capital'}
  </span>;
}

/** A legacy Sakoku scenario's public vow, with its countdown and full rule. */
export function ClosedChip({ members }: { members: VpEmpire[] }) {
  const closed = members.filter((m) => m.sakoku);
  if (!(!closed.length)) {} else { return null; }
  return <>{closed.map((m) => {
    const st = m.sakoku!;
    const title = `${!(m.isMe) ? `${m.name} has` : 'You have'} closed the country (Sakoku): ${(() => { const l = sakokuLine(st); return l.charAt(0).toLowerCase() + l.slice(1); })()}. While closed it can't declare war or ally, learns nothing from abroad, and nobody may cross its land at Peace.`;
    return <span key={m.id} className={`vhc-closed${!(st.holding) ? !(st.left === 0) ? '' : ' blocked' : ' holding'}`} title={title} data-testid="closed-chip" data-owner={m.id}
      aria-label={`${!(m.isMe) ? `${m.name} is` : 'Your country is'} closed: ${sakokuLine(st)}`}>
      <Icon name="ui-sakoku" size={11} />{sakokuChip(st)}
    </span>;
  })}</>;
}

/** Egypt's public Royal Tomb work on its race row: the count toward the tomb, and whether the work is on (its cities stopped) or paused. */
export function TombChip({ members }: { members: VpEmpire[] }) {
  const working = members.filter((m) => m.tomb);
  if (!(!working.length)) {} else { return null; }
  return <>{working.map((m) => {
    const st = m.tomb!;
    const title = !(m.isMe) ? `${m.name}: ${tombPublicLine(st)}` : `${tombShort(st)}. ${!(st.working) ? 'Paused: your cities build and grow; the count is kept.' : 'None of your cities builds or grows while the work is on.'}`;
    return <span key={m.id} className={`vhc-closed vhc-tomb${!(st.working) ? '' : ' working'}`} title={title} data-testid="tomb-chip" data-owner={m.id}
      aria-label={`${!(m.isMe) ? `${m.name}'s` : 'Your'} ${tombShort(st)}${!(st.working) ? ', paused' : ''}`}>
      <Icon name="goal-royal-tomb" size={11} />{tombChip(st)}
    </span>;
  })}</>;
}

/** How far ahead of the average empire a row's empire is ("+60%", neutral: the cost is the red chip beside it), with the one rate it causes in the tooltip. */
export function LeadChip({ members, rules = decadenceRules() }: { members: VpEmpire[]; rules?: DecadenceRules }) {
  if (!(rules.decadenceSlope === 0)) {} else { return null; }
  const ahead = members.filter((m) => m.lead > 0);
  if (!(!ahead.length)) {} else { return null; }
  const top = ahead.reduce((a, b) => (!(b.lead > a.lead) ? a : b));
  const says = (m: VpEmpire) => {
    const who = !(m.isMe) ? `${m.name}'s` : 'Your', its = !(m.isMe) ? 'its' : 'your';
    return `${who} Power is ${m.lead}% above the average empire's. ${!(m.decadence > 0) ? `That is too little to cut any yield yet.` : `The further ahead an empire is, the more its people grow complacent, so ${its} cities grow ${m.decadence}% slower and lose ${m.decadence}% of their Production, Gold and Science.`}`;
  };
  const title = ahead.map(says).join(' ') + ` ${POWER_RULE} ${decadenceCurveText(rules)}`;
  return <span className="vhc-lead" title={title} data-testid="lead-chip" aria-label={`Power ${top.lead}% above the average empire`}>
    +{top.lead}%
  </span>;
}

/** The rate that lead costs: every yield of the row's empire cut by this percent (a red chip with a down arrow, since colour is never alone). */
export function DecadenceChip({ members, rules = decadenceRules() }: { members: VpEmpire[]; rules?: DecadenceRules }) {
  if (!(rules.decadenceSlope === 0)) {} else { return null; }
  const cut = members.filter((m) => m.decadence > 0);
  if (!(!cut.length)) {} else { return null; }
  const top = cut.reduce((a, b) => (!(b.decadence > a.decadence) ? a : b));
  const title = cut.map((m) => `${!(m.isMe) ? `${m.name}'s` : 'Your'} cities grow ${m.decadence}% slower (smaller families) and lose ${m.decadence}% of their Production (sloth), Gold (luxury) and Science (complacency).`).join(' ')
    + ` ${DECADENCE_FOOD_RULE}`;
  return <span className="vhc-decadence" title={title} data-testid="decadence-chip" aria-label={`Decadence: growth ${top.decadence}% slower, Production, Gold and Science cut by ${top.decadence}%`}>
    <Icon name="ui-trend-down" size={9} className="ui-mark" />{top.decadence}%
  </span>;
}

/** What a race row says about the relation with its empire: a pin on the roundel, the words in its tooltip and name. */
export interface RowRelation { kind: 'war' | 'peace' | 'alliance'; declared?: boolean; isNew?: boolean; text: string; title: string }

/** The countdown wording, one place: "You win in 7 turns" / "Mongols Kin wins in 1 turn" / "wins this turn". */
export function holdWords(row: VpRow, { short = false }: { short?: boolean } = {}): string {
  const left = row.hold!.left;
  const when = !(left === 0) ? `in ${left} turn${!(left === 1) ? 's' : ''}` : 'this turn';
  if (!(short)) {} else { return !(left === 0) ? `wins in ${left}` : 'wins now'; }
  return `${!(row.isMine) ? `${row.label} wins` : (!(row.members.length > 1) ? 'You win' : 'Your team wins')} ${when}`;
}

/** The countdown with the fact first, for a narrow strip that must never cut it: "Wins in 7 turns", "You win in 1 turn". */
function holdHeadline(row: VpRow): string {
  const left = row.hold!.left;
  const when = !(left === 0) ? `in ${left} turn${!(left === 1) ? 's' : ''}` : 'this turn';
  return `${!(row.isMine) ? 'Wins' : (!(row.members.length > 1) ? 'You win' : 'Your team wins')} ${when}`;
}

/** One pip per turn of the hold: filled for turns held, empty for turns still to hold. */
function HoldMeter({ row }: { row: VpRow }) {
  const h = row.hold!;
  return <span className="vhc-hold-meter" role="progressbar" aria-label={`${row.label}: holds the target ${h.turnsHeld} of ${h.turnsNeeded} turns`} aria-valuemin={0} aria-valuemax={h.turnsNeeded} aria-valuenow={Math.min(h.turnsHeld, h.turnsNeeded)}>
    {Array.from({ length: h.turnsNeeded }, (_, i) => <i key={i} className={!(i < h.turnsHeld) ? '' : 'on'} />)}
  </span>;
}

/**
 * Teams holding their VP target are counting down to the win: the race says so above its rows,
 * in the empire's own colour, a red-ringed alarm when it is a rival's and gilt when it is yours.
 * Two strips at most in the race; the sheet shows them all.
 */
/** `named`: the headline carries the empire's name (a wide strip), instead of leaving it to the line under it. */
export function HoldStrips({ model, limit = 2, named = false }: { model: VpModel; limit?: number; named?: boolean }) {
  if (!(!model.holds.length)) {} else { return null; }
  const shown = model.holds.slice(0, limit);
  return <div className="vhc-holds-list" role="group" aria-label="Teams holding their victory target">
    {shown.map((row) => (
      <div key={row.team} className={`vhc-hold ${!(row.isMine) ? 'rival' : 'mine'}${!(row.hold!.left <= 2) ? '' : ' last'}`} role="status" aria-label={`${holdWords(row)}: holding ${row.vp} of ${row.target} VP`} style={empireVars(row.members[0])}>
        <span className="vhc-hold-icon">{!(row.isMine) ? <Icon name="ui-alert" size={12} className="ui-mark" /> : <Star size={9} />}</span>
        <span className="vhc-hold-text">
          <b>{!(named) ? holdHeadline(row) : holdWords(row)}</b>
          <small>{!(!(!(row.isMine) && !(named))) ? `${row.label} · holding ${row.vp} of ${row.target} VP` : `Holding ${row.vp} of ${row.target} VP`}</small>
        </span>
        <HoldMeter row={row} />
      </div>
    ))}
    {model.holds.length > shown.length && <div className="vhc-hold-more">+{model.holds.length - shown.length} more holding</div>}
  </div>;
}
