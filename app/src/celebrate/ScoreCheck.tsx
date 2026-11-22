import { useEffect, useLayoutEffect, useRef } from 'react';
import { feel, useCombatFx } from '../feel';
import { CrestMedallion } from '../brand/ornaments';
import { Icon } from '../ui/bits';
import { useReducedMotion } from '../ui/useReducedMotion';
import { eraArt } from './eraArt';
import type { Moment } from './moments';
import type { Box, ScoreCheckCard } from './scoreCheckCards';
import './scoreCheck.css';

const LEGEND: { state: Box['state']; text: string }[] = [
  { state: 'kept', text: 'Scored, kept for good' },
  { state: 'held', text: 'Held: yours only while you keep it' },
  { state: 'new', text: 'New this Age' },
  { state: 'open', text: 'Still to win' },
  { state: 'later', text: 'Age to come' },
];

const CREST = 32;
const BOX = 24, GAP = 4;
/** The capital star the map draws (`CapitalStarGlyph`'s filled path). Always filled: on the map a hollow star means "its VP is yours", so a capital still to take is a faint filled star (art review, 2026-11-22). */
const STAR = 'M0 -5.4L1.6 -1.6L5.4 0L1.6 1.6L0 5.4L-1.6 1.6L-5.4 0L-1.6 -1.6Z';

/** What a box shows: the capital's star, the wonder's or goal's icon, or the Age's numeral. */
function BoxFace({ box }: { box: Box }) {
  if (box.star) return <svg width={14} height={14} viewBox="-6 -6 12 12" aria-hidden="true"><path className="sc-star" d={STAR} fill="currentColor" stroke="currentColor" strokeWidth={0.6} strokeLinejoin="round" /></svg>;
  if (box.icon) return <Icon name={box.icon} size={16} />;
  return <>{box.mark}</>;
}

/**
 * The tally (premium pass, user 2026-11-22): every empire's crest walks up the track from 0 to its VP, one space at a
 * time like a pawn, then your boxes ink in one by one and the new Age's card glows in last. About two seconds; none
 * under reduced motion or with animations off; a click on the sheet or track finishes it at once, and Continue works
 * throughout. Web Animations only, so the finished layout is the one the tests measure.
 */
function useTally(root: React.RefObject<HTMLDivElement | null>, id: string, off: boolean) {
  const running = useRef<Animation[]>([]);
  useLayoutEffect(() => {
    const el = root.current;
    if (off || !el) return;
    const zero = el.querySelector<HTMLElement>('[data-testid=score-track] li[data-vp="0"] .sc-pieces');
    const animations: Animation[] = [];
    const animate = (node: HTMLElement, frames: Keyframe[], options: KeyframeAnimationOptions) => {
      if (typeof node.animate !== 'function') return;
      const a = node.animate(frames, { fill: 'backwards', ...options });
      a.finished?.catch(() => undefined); // cancelled on close: not an error
      animations.push(a);
    };
    const from = zero?.getBoundingClientRect();
    let landed = 0;
    el.querySelectorAll<HTMLElement>('.sc-piece').forEach((piece, i) => {
      const vp = Number(piece.dataset.vp);
      if (!from || !vp) return;
      const at = piece.getBoundingClientRect();
      const dx = from.left + from.width / 2 - (at.left + at.width / 2);
      // One hop per space, so it reads as counting: down on each space, up between them.
      const frames: Keyframe[] = [{ transform: `translate(${dx}px, 0)`, opacity: 0 }];
      for (let k = 1; k <= vp; k++) {
        const x = (dx * (vp - k + 0.5)) / vp;
        frames.push({ transform: `translate(${x}px, -7px)`, opacity: 1, offset: (k - 0.5) / vp, easing: 'ease-in' });
        frames.push({ transform: `translate(${(dx * (vp - k)) / vp}px, 0)`, opacity: 1, offset: k / vp, easing: 'ease-out' });
      }
      const step = Math.max(70, Math.min(140, 1000 / vp));
      const delay = 420 + i * 45, duration = step * vp;
      animate(piece, frames, { duration, delay, easing: 'linear' });
      landed = Math.max(landed, delay + duration);
      // Your own crest gets its landing beat: a gilt flare as it arrives.
      const crest = piece.firstElementChild as HTMLElement | null;
      if (piece.classList.contains('mine') && crest) {
        animate(crest, [{ transform: 'scale(1)', filter: 'drop-shadow(0 0 0 rgb(236 196 108 / 0))' }, { transform: 'scale(1.22)', filter: 'drop-shadow(0 0 10px rgb(236 196 108 / 1))', offset: 0.4 }, { transform: 'scale(1)', filter: 'drop-shadow(0 0 0 rgb(236 196 108 / 0))' }],
          { duration: 480, delay: delay + duration, easing: 'ease-out', fill: 'none' });
      }
    });
    const stampsFrom = Math.max(760, Math.min(landed - 200, 1300));
    el.querySelectorAll<HTMLElement>('.sc-sheet .sc-row .sc-box.kept, .sc-sheet .sc-row .sc-box.held').forEach((box, i) => {
      animate(box, [{ transform: 'scale(1.7)', opacity: 0 }, { transform: 'scale(1)', opacity: 1 }], { duration: 240, delay: stampsFrom + i * 80, easing: 'cubic-bezier(.2,.9,.3,1.25)' });
      // A soft tick as each box inks in (the sound and haptic settings apply; the feel layer spaces them).
      animations[animations.length - 1]?.finished?.then(() => feel.snap(), () => undefined);
    });
    const inked = el.querySelectorAll('.sc-sheet .sc-row .sc-box.kept, .sc-sheet .sc-row .sc-box.held').length;
    el.querySelectorAll<HTMLElement>('.sc-sheet .sc-row .sc-box.new').forEach((box) => {
      animate(box, [{ transform: 'scale(.6)', opacity: 0, boxShadow: '0 0 0 rgb(236 196 108 / 0)' }, { transform: 'scale(1.15)', opacity: 1, boxShadow: '0 0 18px rgb(236 196 108 / 1)', offset: 0.6 }, { transform: 'scale(1)', opacity: 1 }],
        { duration: 520, delay: stampsFrom + 100 + inked * 80, easing: 'ease-out' });
    });
    running.current = animations;
    return () => { animations.forEach((a) => a.cancel()); running.current = []; };
  }, [root, id, off]);
  return () => running.current.forEach((a) => a.finish());
}

/**
 * The Age score check (legatum-ui "Big moments"): when a World Age begins, every player gets this takeover, after their own
 * era-up if they opened the Age. The score track shows who is ahead; your sheet shows every way to score as boxes that
 * draw the thing itself (an Age's numeral, a capital's star, a wonder's or goal's icon), with how to win the next point.
 * It is held like the era-up: Continue, Return or Esc close it, nothing else does.
 */
export function ScoreCheck({ moment, dismiss }: { moment: Moment; dismiss(): void }) {
  const card: ScoreCheckCard = moment.scoreCheck!;
  const art = eraArt(card.era - 1);
  const motionOff = useReducedMotion();
  const animationsOn = useCombatFx((s) => s.enabled);
  const reduced = motionOff || !animationsOn;
  const root = useRef<HTMLDivElement>(null);
  const primary = useRef<HTMLButtonElement>(null);
  useEffect(() => { primary.current?.focus({ preventScroll: true }); }, [moment.id]);
  const finish = useTally(root, moment.id, reduced);
  return <div ref={root} key={moment.id} className={`era-takeover score-check ${reduced ? 'reduced' : ''}`} role="dialog" aria-modal="true" aria-label={`Score check: the ${card.name} Age begins`}
    data-testid="score-check" data-era={card.era}>
    <div className="et-art" aria-hidden="true">
      {art && <img src={art.small} srcSet={`${art.small} 1280w, ${art.large} 1920w`} sizes="100vw" alt="" draggable={false} />}
    </div>
    <div className="sc-wash" aria-hidden="true" />
    <div className="sc-body" onPointerDown={finish}>
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
              {s.pieces.map((p) => <span key={p.id} className={`sc-piece${p.isMine ? ' mine' : ''}`} data-vp={s.vp} title={`${p.name}: ${s.vp} VP`}><CrestMedallion faction={p.faction} color={p.color} size={CREST} /></span>)}
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
          <span className="sc-boxes">{l.boxes.map((b, i) => <span key={i} className={`sc-cell ${b.state}`}><span className={`sc-box ${b.state}`} title={b.title} aria-label={b.title} data-span={b.span ?? 1}
            style={b.span && b.span > 1 ? { width: BOX * b.span + GAP * (b.span - 1) } : undefined}><BoxFace box={b} /></span></span>)}</span>
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
