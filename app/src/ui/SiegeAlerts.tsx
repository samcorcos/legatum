import { useGame } from '../store';
import type { CityAlarm } from '../siege';
import { feel } from '../feel';
import { Icon } from './bits';
import { Tip } from './Tip';
import { useCityAlarms } from './useCityAlarms';
import './siege.css';

/** The most cards shown in full; any others are one line each, still one click from their city. */
const FULL_CARDS = 3;

/** What the attackers are doing, in one line, with the damage they do: "3 Spearmen beside it · about 84 HP a turn". */
function whoLine(a: CityAlarm): string {
  if (a.kind === 'approach') return `${a.force} from ${a.from} within 3 tiles`;
  if (a.attackers.length) {
    const more = a.approaching.length ? ` · ${a.approaching.length} more within 3 tiles` : '';
    return `${a.force} from ${a.from}: about ${a.perTurn} HP a turn${more}`;
  }
  return 'Hit last turn; nothing can strike it next turn';
}

/** "−80 last turn", or the plain fact when only the city's own record is left (a loaded game). */
function lostText(a: CityAlarm): string | null {
  if (a.hit) return `−${a.hit.lost} HP last turn`;
  return a.attackedLastTurn ? 'Took damage last turn' : null;
}

function SiegeCard({ a }: { a: CityAlarm }) {
  const select = useGame((s) => s.select);
  const go = () => { select({ kind: 'city', id: a.cityId }, true); feel.select(); };
  const lost = lostText(a);
  const fill = Math.max(0, Math.min(100, (a.hp / a.maxHp) * 100));
  const before = a.hit ? Math.max(0, Math.min(100, (a.hit.before / a.maxHp) * 100)) : fill;
  const title = a.kind === 'approach' ? `Army closing on ${a.name}` : `${a.name} is under attack`;
  const approachingWhen = a.kind === 'approach'
    ? (a.forecast.text.includes('next turn') ? '1 turn away' : '2 turns away')
    : null;
  const compactForecast = a.hp <= 0 ? a.forecast.text
    : a.forecast.turns === 1 ? 'Falls next turn'
    : a.forecast.turns !== null ? `Falls in ${a.forecast.turns} turns`
      : a.forecast.text;
  return (
    <section className={`siege-card ${a.kind} ${a.forecast.level}`} aria-label={title} data-city={a.cityId} data-level={a.forecast.level} data-kind={a.kind}>
      <header className="siege-head">
        <span className="siege-mark" aria-hidden="true"><Icon name={a.kind === 'attack' ? 'ui-siege' : 'ui-alert'} size={20} /></span>
        <h4 className="siege-title">{title}</h4>
        {a.kind === 'attack' && <span className="siege-empire"> · {a.from}</span>}
        {approachingWhen && <span className="siege-approach-time"> · {approachingWhen}</span>}
      </header>
      {a.kind === 'attack' && <div className="siege-hp" role="img" aria-label={`${a.hp} of ${a.maxHp} HP${a.hit ? `, ${a.hit.lost} lost last turn` : ''}`}>
        <span className="siege-bar" aria-hidden="true">
          <span className="siege-lostbar" style={{ width: `${before}%` }} />
          <span className="siege-fillbar" style={{ width: `${fill}%` }} />
        </span>
        <b className="siege-num">{a.hp}<span className="siege-of">/{a.maxHp} HP</span></b>
        {lost && <b className="siege-lost">{lost}</b>}
      </div>}
      <Tip content={a.forecast.detail} className="siege-forecast-tip" placement="bottom">
        <span className="siege-forecast"><Icon name="ui-alert" size={14} className="ui-mark" />{a.forecast.text}</span>
      </Tip>
      <div className="siege-foot">
        <span className="siege-who">{whoLine(a)}</span>
        <span className="siege-compact-who">{a.kind === 'attack' ? a.from : whoLine(a)}</span>
        <button className="btn siege-go" onClick={go} aria-label={`Go to ${a.name}`}><span className="siege-go-full">Go to city</span><span className="siege-go-short">Go</span></button>
      </div>
      {a.kind === 'attack' && <p className="siege-compact-facts">
        <b>{a.hp}/{a.maxHp} HP</b>{lost && <> · {lost}</>} · {compactForecast}
      </p>}
    </section>
  );
}

/**
 * The top of This Turn (user, 2026-10-08: "I shouldn't be surprised to see that one of my cities
 * is about to get taken over"): every city under attack or about to be, with its HP, the attackers,
 * a capture forecast from public numbers and a button that goes there. Above everything routine.
 */
export function SiegeAlerts() {
  const alarms = useCityAlarms();
  const select = useGame((s) => s.select);
  if (!alarms.length) return null;
  const full = alarms.slice(0, FULL_CARDS), rest = alarms.slice(FULL_CARDS);
  const attacked = alarms.filter((a) => a.kind === 'attack').length;
  return (
    <div className="siege-alerts" role="region" aria-label={attacked ? 'Cities under attack' : 'Armies approaching'} data-testid="siege-alerts">
      {full.map((a) => <SiegeCard key={`${a.kind}-${a.cityId}`} a={a} />)}
      {rest.length > 0 && <div className="siege-rest">
        {rest.map((a) => <button key={`${a.kind}-${a.cityId}`} className="link" onClick={() => select({ kind: 'city', id: a.cityId }, true)}>
          {a.kind === 'attack' ? `${a.name} is under attack` : `Army closing on ${a.name}`} · {a.forecast.text}
        </button>)}
      </div>}
    </div>
  );
}
