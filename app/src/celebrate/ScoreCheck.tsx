import { useEffect, useRef } from 'react';
import { feel } from '../feel';
import { CrestMedallion } from '../brand/ornaments';
import { eraArt } from './eraArt';
import type { Moment } from './moments';
import type { Box, ScoreCheckCard } from './scoreCheckCards';
import './scoreCheck.css';

const LEGEND: { state: Box['state']; text: string }[] = [
  { state: 'kept', text: 'Scored, kept for good' },
  { state: 'held', text: 'Held: you lose it if you lose the city or goal' },
  { state: 'new', text: 'New this Age' },
  { state: 'open', text: 'Still to win' },
  { state: 'later', text: 'Age to come' },
];

/**
 * The Age score check (legatum-ui "Big moments"): when a World Age begins, every player gets this takeover, after their own
 * era-up if they opened the Age. The score track shows who is ahead; your sheet shows every way to score as boxes, with
 * how to win the next point. It is held like the era-up: Continue, Return or Esc close it, nothing else does.
 */
export function ScoreCheck({ moment, dismiss }: { moment: Moment; dismiss(): void }) {
  const card: ScoreCheckCard = moment.scoreCheck!;
  const art = eraArt(card.era - 1);
  const primary = useRef<HTMLButtonElement>(null);
  useEffect(() => { primary.current?.focus({ preventScroll: true }); }, [moment.id]);
  return <div key={moment.id} className="era-takeover score-check" role="dialog" aria-modal="true" aria-label={`Score check: the ${card.name} Age begins`}
    data-testid="score-check" data-era={card.era}>
    <div className="et-art" aria-hidden="true">
      {art && <img src={art.small} srcSet={`${art.small} 1280w, ${art.large} 1920w`} sizes="100vw" alt="" draggable={false} />}
    </div>
    <div className="sc-wash" aria-hidden="true" />
    <div className="sc-body">
      <header className="sc-head">
        <div>
          <div className="sc-kicker">{card.replay ? `The ${card.name} Age · the score now · Turn ${card.turn}` : `The ${card.name} Age begins · Turn ${card.turn}`}</div>
          <h2 className="sc-title">Score check</h2>
        </div>
        <p className="sc-rule">{card.opener}<br />First to hold <b>{card.target} VP</b> for <b>10 turns</b> wins.</p>
      </header>

      <section className="sc-panel sc-track" aria-label="Who is ahead" data-testid="score-track">
        <ol style={{ gridTemplateColumns: `repeat(${card.track.length}, minmax(0, 1fr))` }}>
          {card.track.map((s) => <li key={s.vp} className={`${s.pieces.some((p) => p.isMine) ? 'mine' : ''} ${s.vp === card.target ? 'win' : ''}`} data-vp={s.vp}>
            <span className="sc-pieces">
              {s.pieces.map((p) => <span key={p.id} className={`sc-piece${p.isMine ? ' mine' : ''}`} title={`${p.name}: ${s.vp} VP`}><CrestMedallion faction={p.faction} color={p.color} size={24} /></span>)}
            </span>
            <span className="sc-space">{s.vp === card.target ? `${s.vp} · Win` : s.vp}</span>
          </li>)}
        </ol>
        <p className="sc-standing"><span>{card.standing}</span><span>{card.leader}</span></p>
        {card.hold && <p className="sc-hold" role="alert">{card.hold}</p>}
      </section>

      <section className="sc-panel sc-sheet" aria-label="Your score sheet" data-testid="score-sheet">
        <div className="sc-row sc-cols" aria-hidden="true"><span>Your sheet</span><span>Scored and open</span><span>How to score more</span><span>VP left</span></div>
        {card.lines.map((l) => <div key={l.key} className={`sc-row${l.fresh ? ' fresh' : ''}`} data-line={l.key}>
          <span className="sc-name"><b>{l.name}</b><small>{l.terms}</small></span>
          <span className="sc-boxes">{l.boxes.map((b, i) => <span key={i} className={`sc-box ${b.state}`} title={b.title} aria-label={b.title}>{b.mark}</span>)}</span>
          <span className="sc-how"><b>{l.how}</b>{l.detail && <small title={l.detail}>{l.detail}</small>}</span>
          <span className="sc-open">{l.open > 0 ? `+${l.open}` : '–'}</span>
        </div>)}
        <ul className="sc-legend" aria-label="Key">{LEGEND.map((k) => <li key={k.state}><span className={`sc-box ${k.state}`} aria-hidden="true" />{k.text}</li>)}</ul>
      </section>

      <footer className="et-foot sc-foot">
        <span />
        <button ref={primary} className="btn primary" data-primary-action onClick={() => { feel.select(); dismiss(); }}>Continue <span className="kbd">Return</span></button>
      </footer>
    </div>
  </div>;
}
