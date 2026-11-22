// In-play scenario UI: the objective bar (always visible: what to do, how many turns are left,
// the live score and each condition) and the result screen when the scenario ends.
import { useId, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react';
import { useGame, type ActiveScenario } from '../store';
import { Star, Wordmark } from '../brand/ornaments';
import { PortraitFrame, Ribbon } from './premium';
import { factionColor } from '../theme';
import { FELT_CHOICES, OUTCOME_TITLE, compareToAi, leadTarget, targetText, turnsText, type FeltLike, type ScenarioTarget } from '../scenarios/scenarios';
import { closeTips } from './Tip';
import { Icon, NotMetMark } from './bits';
import './Scenarios.css';

function Conditions({ sc }: { sc: ActiveScenario }) {
  return (
    <ul className="scn-conditions">
      {sc.status.conditions.map((c) => (
        <li key={c.label} className={c.met ? 'met' : 'unmet'}>
          <span className="scn-mark" aria-hidden>{c.met ? <Icon name="ui-check" size={12} /> : <NotMetMark size={12} />}</span>
          <span>{c.label}{c.keep ? ' (every turn)' : ''}</span>
          <b>{c.progress}</b>
        </li>
      ))}
    </ul>
  );
}

function ScoreTable({ sc }: { sc: ActiveScenario }) {
  return (
    <table className="scn-score-table">
      <tbody>
        {sc.status.score_lines.map((l) => (
          <tr key={l.label}>
            <td>{l.label}</td>
            <td className="num">{l.value} × {l.weight}</td>
            <td className={`num ${l.points < 0 ? 'neg' : ''}`}>{l.points > 0 ? '+' : ''}{l.points}</td>
          </tr>
        ))}
        <tr className="total">
          <td>Score</td>
          <td />
          <td className="num">{sc.status.score}</td>
        </tr>
      </tbody>
    </table>
  );
}

/** Every city the objective is about, each with its own "Show on map". */
function Targets({ sc }: { sc: ActiveScenario }) {
  const jumpTo = useGame((s) => s.jumpTo);
  const targets = sc.status.targets ?? [];
  if (!targets.length) return null;
  return (
    <ul className="scn-targets" data-testid="scenario-targets">
      {targets.map((t) => (
        <li key={t.city} className={`kind-${t.kind.toLowerCase()}`}>
          <TargetMark kind={t.kind} />
          <span>{targetText(t)}</span>
          <button className="link" onClick={() => jumpTo(t.pos)}>Show on map</button>
        </li>
      ))}
    </ul>
  );
}

/** The same marks the map draws on target cities: crosshair (take), shield (hold), ring (contain). */
function TargetMark({ kind }: { kind: ScenarioTarget['kind'] }) {
  return (
    <svg className="scn-target-mark" viewBox="0 0 16 16" aria-hidden width={14} height={14}>
      {kind === 'Take' && <g fill="none" stroke="currentColor" strokeWidth="1.6"><circle cx="8" cy="8" r="5" /><path d="M8 1v4M8 11v4M1 8h4M11 8h4" /></g>}
      {kind === 'Hold' && <path d="M8 1.5l5.5 2v4c0 3.4-2.4 5.8-5.5 7-3.1-1.2-5.5-3.6-5.5-7v-4z" fill="none" stroke="currentColor" strokeWidth="1.6" />}
      {kind === 'Contain' && <circle cx="8" cy="8" r="5.5" fill="none" stroke="currentColor" strokeWidth="1.6" strokeDasharray="3 2" />}
    </svg>
  );
}

/** The objective bar under the top bar. Click it for the conditions and how the score adds up. */
export function ScenarioBar() {
  const sc = useGame((s) => s.scenario);
  const set = useGame((s) => s.set);
  const jumpTo = useGame((s) => s.jumpTo);
  const [open, setOpen] = useState(false);
  if (!sc) return null;
  const st = sc.status;
  const over = st.outcome !== 'Playing';
  const lead = leadTarget(st.targets, `${sc.entry.file.objective.summary} ${sc.entry.file.briefing}`);
  return (
    <div className={`scenario-bar ${over ? 'over' : ''} ${open ? 'open' : ''}`} data-overlay-slot="scenario-bar" data-testid="scenario-bar">
      <button className="sb-main" onClick={() => setOpen(!open)} aria-expanded={open} aria-controls="scenario-details">
        <span className="sb-kicker lbl">{sc.entry.file.title}</span>
        <span className="sb-objective">{sc.entry.file.objective.summary}</span>
        <span className="sb-stats">
          <span className={`sb-turns ${!over && st.turns_left <= 2 ? 'urgent' : ''}`} data-testid="scenario-turns">{turnsText(st)}</span>
          <span className="sb-score" title="Your scenario score">Scenario score <b>{st.score}</b></span>
          <span className="sb-more" aria-hidden><Icon name={open ? 'ui-chevron-up' : 'ui-chevron-down'} size={12} /></span>
        </span>
      </button>
      {lead && !over && (
        <div className={`sb-target kind-${lead.kind.toLowerCase()}`} data-testid="scenario-target">
          <TargetMark kind={lead.kind} />
          <span className="sb-target-text">{targetText(lead)}</span>
          <button className="btn small sb-show" onClick={() => jumpTo(lead.pos)} data-testid="scenario-show-target">
            Show on map
          </button>
        </div>
      )}
      {over && (
        <button className="btn sb-result" onClick={() => set({ scenario: { ...sc, resultOpen: true } })}>
          See result
        </button>
      )}
      {open && (
        <div id="scenario-details" className="sb-details">
          <div className="sb-window">Turn {st.turn} · the window ends on turn {st.end_turn}</div>
          <p className="sb-briefing">{sc.entry.file.briefing}</p>
          <Targets sc={sc} />
          <Conditions sc={sc} />
          <ScoreTable sc={sc} />
        </div>
      )}
    </div>
  );
}

function SavedLine({ sc }: { sc: ActiveScenario }) {
  const saved = sc.saved;
  if (!saved) return <p className="scn-saved" data-testid="scenario-saved">Saving your run…</p>;
  if ('error' in saved) return <p className="scn-saved" data-testid="scenario-saved">Your run could not be saved: {saved.error}</p>;
  return (
    <p className="scn-saved" data-testid="scenario-saved">
      Your run was downloaded: <span className="scn-file">{saved.path}</span>
    </p>
  );
}

/** Report categories in the order they matter to a scenario: cities change hands first. */
const TIMELINE_ORDER = ['cities', 'combat', 'diplomacy', 'victory', 'economy', 'science', 'discoveries', 'orders'];

/** What happened: each turn's most important report lines for the seat, oldest turn first. */
function Timeline({ sc }: { sc: ActiveScenario }) {
  const rank = (category: string) => { const i = TIMELINE_ORDER.indexOf(category); return i < 0 ? TIMELINE_ORDER.length : i; };
  const lines = sc.run.turns.flatMap((t) => t.events
    .filter((e) => e.weight >= 2 && e.mine)
    .map((e, i) => ({ ...e, turn: t.turn, i }))
    .sort((a, b) => rank(a.category) - rank(b.category) || a.i - b.i)
    .slice(0, 4));
  if (!lines.length) return <p className="scn-quiet">No battles, captures or treaties touched your empire.</p>;
  return (
    <ol className="scn-timeline">
      {lines.slice(-12).map((l, i) => (
        <li key={i} className={`tone-${l.tone}`}>
          <span className="t">T{l.turn}</span>
          <span>{l.text}</span>
        </li>
      ))}
    </ol>
  );
}

/** One 1 to 5 answer: a row of numbered buttons with the two ends named. */
function Rating({ label, low, high, value, onChange }: { label: string; low: string; high: string; value: number | null; onChange: (n: number) => void }) {
  return (
    <div className="scn-rating" role="radiogroup" aria-label={label}>
      <span className="lbl">{label}</span>
      <span className="scn-rating-row" onKeyDown={(e) => {
        if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return;
        e.preventDefault();
        const next = Math.min(5, Math.max(1, (value ?? (e.key === 'ArrowRight' ? 0 : 6)) + (e.key === 'ArrowRight' ? 1 : -1)));
        onChange(next);
        (e.currentTarget.children[next] as HTMLElement | undefined)?.focus();
      }}>
        <span className="scn-rating-end">{low}</span>
        {[1, 2, 3, 4, 5].map((n) => (
          <button key={n} type="button" role="radio" aria-checked={value === n} tabIndex={value === n || (value === null && n === 1) ? 0 : -1} className={`scn-pick ${value === n ? 'on' : ''}`} onClick={() => onChange(n)}>{n}</button>
        ))}
        <span className="scn-rating-end">{high}</span>
      </span>
    </div>
  );
}

type Step = 'result' | 'feedback' | 'thanks';

export function ScenarioResult() {
  const sc = useGame((s) => s.scenario);
  const set = useGame((s) => s.set);
  const goto = useGame((s) => s.goto);
  const startScenario = useGame((s) => s.startScenario);
  const submit = useGame((s) => s.submitScenarioFeedback);
  const dialog = useRef<HTMLDialogElement>(null);
  const primary = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const visible = !!sc?.resultOpen;
  const [step, setStep] = useState<Step>('result');
  const [fun, setFun] = useState<number | null>(null);
  const [fair, setFair] = useState<number | null>(null);
  const [felt, setFelt] = useState<FeltLike | null>(null);
  const [text, setText] = useState('');
  const [typing, setTyping] = useState(false);
  const hasFeedback = !!sc?.run.feedback;
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!visible || !el) return;
    closeTips();
    setStep(hasFeedback ? 'thanks' : 'result');
    el.showModal();
    primary.current?.focus({ preventScroll: true });
    return () => el.close();
    // Opening is what resets the card; a saved answer arriving later must not.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [visible]);
  useLayoutEffect(() => {
    if (visible) primary.current?.focus({ preventScroll: true });
  }, [step, visible]);
  if (!sc || !visible) return null;
  const f = sc.entry.file;
  const st = sc.status;
  const lab = f.lab;
  const close = () => set({ scenario: { ...sc, resultOpen: false } });
  const tone = st.outcome === 'Won' || st.outcome === 'Complete' ? 'gilt' : 'red';
  const word = st.outcome === 'Won' ? 'Met' : st.outcome === 'Complete' ? 'Complete' : 'Failed';
  const cmp = compareToAi(f, st);
  const p = f.predictions;
  const answered = fun !== null || fair !== null || felt !== null || text.trim() !== '';
  const save = () => {
    if (!answered) return;
    void submit({ fun, fair, felt, text: text.trim() });
    setStep('thanks');
  };
  const onKeyDown = (e: KeyboardEvent<HTMLDialogElement>) => {
    e.stopPropagation();
    if (step !== 'feedback' || e.key !== 'Enter') return;
    const inText = e.target instanceof HTMLTextAreaElement;
    // Return saves from anywhere but the text box (where it is a new line, and Cmd or Ctrl+Return saves);
    // on a Skip or Save button it keeps its own meaning, on a rating it saves (Space picks).
    const onOtherButton = e.target instanceof HTMLButtonElement && !e.target.classList.contains('scn-pick');
    if (inText ? e.metaKey || e.ctrlKey : !onOtherButton) {
      e.preventDefault();
      save();
    }
  };
  return (
    <dialog ref={dialog} className={`modal gameover scn-result ${st.outcome === 'Won' ? 'won' : st.outcome === 'Lost' ? 'lost' : 'draw'}`}
      aria-labelledby={titleId} data-testid="scenario-result" data-step={step}
      onCancel={(e) => { e.preventDefault(); if (step === 'feedback') setStep('result'); else close(); }}
      onKeyDown={onKeyDown}>
      <Wordmark height={18} />
      <div className="go-kicker"><Star size={8} /> {f.title} · turn {st.turn} <Star size={8} /></div>
      {step === 'result' && (
        <>
          <div className="scn-hero">
            <PortraitFrame faction={f.faction} crestColor={factionColor(f.faction)} fallen={tone === 'red'} />
            <Ribbon tone={tone}>{word}</Ribbon>
          </div>
          <h1 id={titleId}>{OUTCOME_TITLE[st.outcome]}</h1>
          <p>{st.reason}</p>
          <div className="scn-vs" data-testid="scenario-vs">
            <div className="scn-vs-col you">
              <span className="lbl">You</span>
              <p>{cmp.you}</p>
              <div className="scn-big-score" data-testid="scenario-score"><span className="lbl">Score</span><b>{st.score}</b></div>
            </div>
            <div className="scn-vs-col ai">
              <span className="lbl">Hard AI, same start</span>
              <p>{cmp.ai}</p>
            </div>
          </div>
          {cmp.verdict && <p className="scn-verdict" data-testid="scenario-verdict">{cmp.verdict}</p>}
          <div className="scn-result-actions go-actions">
            <button ref={primary} className="btn primary" onClick={() => setStep('feedback')} data-testid="scenario-feel">How did it feel? <kbd>Return</kbd></button>
            <button className="btn" onClick={() => void startScenario(sc.entry)}>Play again</button>
            <button className="btn quiet" onClick={close}>View the map</button>
          </div>
          <div className="scn-result-body">
            <section>
              <h2 className="lbl">How the score adds up</h2>
              <Conditions sc={sc} />
              <ScoreTable sc={sc} />
            </section>
            <section>
              <h2 className="lbl">What happened</h2>
              <Timeline sc={sc} />
            </section>
            {(p.good_play || p.ai_play) && (
              <details className="scn-predictions">
                <summary>What we expected before you played</summary>
                {p.good_play && <p><b>Good play:</b> {p.good_play}</p>}
                {p.ai_play && <p><b>The Hard AI:</b> {p.ai_play}</p>}
              </details>
            )}
          </div>
        </>
      )}
      {step === 'feedback' && (
        <div className="scn-feedback" data-testid="scenario-feedback">
          <h1 id={titleId}>How did it feel?</h1>
          {lab && <p className="scn-fb-question">{lab.question}</p>}
          <Rating label="Fun" low="Dull" high="Great" value={fun} onChange={setFun} />
          <Rating label="Fair" low="Unfair" high="Fair" value={fair} onChange={setFair} />
          <div className="scn-rating" role="radiogroup" aria-label="Felt like">
            <span className="lbl">It felt</span>
            <span className="scn-felt">
              {FELT_CHOICES.map((c) => (
                <button key={c.id} type="button" role="radio" aria-checked={felt === c.id} className={`scn-pick wide ${felt === c.id ? 'on' : ''}`} onClick={() => setFelt(c.id)}>{c.label}</button>
              ))}
            </span>
          </div>
          <label className="scn-fb-text">
            <span className="lbl">{lab?.ask ?? 'Anything else? What you would change, or what confused you.'}</span>
            <textarea value={text} onChange={(e) => setText(e.target.value)} onFocus={() => setTyping(true)} onBlur={() => setTyping(false)} rows={3} maxLength={2000} placeholder="Type here" />
          </label>
          <div className="scn-result-actions go-actions">
            <button ref={primary} className="btn primary" disabled={!answered} onClick={save} data-testid="scenario-save-feedback">Save feedback <kbd>{typing ? '⌘ Return' : 'Return'}</kbd></button>
            <button className="btn quiet" onClick={() => setStep('thanks')}>Skip</button>
          </div>
        </div>
      )}
      {step === 'thanks' && (
        <div className="scn-thanks">
          <h1 id={titleId}>{hasFeedback ? 'Thank you' : 'Run saved'}</h1>
          <p>{hasFeedback ? 'Your answers are saved with the run, so the scenario can be tuned.' : 'Your run is saved. You can still say how it felt from the result card.'}</p>
          <div className="scn-result-actions go-actions">
            <button ref={primary} className="btn primary" onClick={() => goto('scenarios')}>Back to scenarios <kbd>Return</kbd></button>
            <button className="btn" onClick={() => void startScenario(sc.entry)}>Play again</button>
            <button className="btn quiet" onClick={() => setStep('result')}>See the result</button>
            <button className="btn quiet" onClick={close}>View the map</button>
          </div>
        </div>
      )}
      <SavedLine sc={sc} />
    </dialog>
  );
}
