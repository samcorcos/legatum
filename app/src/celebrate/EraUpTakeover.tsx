import { useEffect, useRef } from 'react';
import { feel, useCombatFx } from '../feel';
import { Icon } from '../ui/bits';
import { useReducedMotion } from '../ui/useReducedMotion';
import { cityStem, cityUrl } from '../map/cityArt';
import type { Moment } from './moments';
import type { EraBring, EraUpCard } from './eraUp';
import { itemArtUrl } from '../unlockArt';

/** A new piece's painted miniature (null while unpainted: the row keeps its icon). */
const bringArt = (b: EraBring) => itemArtUrl(b.item, b.faction, 128);
import { eraArt } from './eraArt';
import { chooseResearch } from './UnlockMoment';
import './eraTakeover.css';

/** The card's primary action (Return): open the research screen when nothing is set, else nothing. */
export function activateEraUp(card: EraUpCard): void {
  if (card.primary === 'research') chooseResearch();
}

/** How many unlock tiles fit the "new this era" column before it says "+N more". */
const BRING_SHOWN = 8;
/** Gold motes drifting up through the painting: fixed positions, so the scene is the same every time. */
const MOTES = Array.from({ length: 18 }, (_, i) => ({
  x: (i * 37 + 11) % 100, size: 2 + ((i * 5) % 4), delay: ((i * 0.37) % 2.4) + 0.4, rise: 160 + ((i * 53) % 220), sway: ((i * 29) % 40) - 20,
}));

/**
 * The viewer's own era-up (legatum-ui "Big moments"): a full-screen takeover with the era's painting, a short animation
 * (under 4 s, and none under reduced motion), what changes (new units and buildings and how your cities now look;
 * the Age's points are the score check that follows) and one primary, Continue. Everything that triggers it lives outside this file:
 * `momentsForTurn` makes a held `Moment` with `eraUp` set from the public `EraEntered` event, and `MomentLane` renders
 * it here and treats it like an unlock card: it waits for Return, Esc or its own buttons, never times out, never
 * touches an order in progress, and never blocks a teammate (it is local to this screen; simultaneous turns go on).
 */
export function EraUpTakeover({ moment, dismiss }: { moment: Moment; dismiss(): void }) {
  const card = moment.eraUp!;
  const motionOff = useReducedMotion();
  const animationsOn = useCombatFx((s) => s.enabled);
  const reduced = motionOff || !animationsOn;
  const art = eraArt(card.era);
  const primary = useRef<HTMLButtonElement>(null);
  useEffect(() => { primary.current?.focus({ preventScroll: true }); }, [moment.id]);
  const shown = card.brings.slice(0, BRING_SHOWN);
  const more = card.brings.length - shown.length;
  const remaining = card.techs.filter((t) => !t.known).length;
  const { tier } = card.cities;
  const to = card.cities.to, from = card.cities.from;
  const urlOf = (era: number) => { const stem = cityStem(era, tier); return stem ? cityUrl(stem, 128) : null; };
  const toUrl = urlOf(to.era), fromUrl = from ? urlOf(from.era) : null;
  return <div key={moment.id} className={`era-takeover ${reduced ? 'reduced' : ''}`} role="dialog" aria-modal="true" aria-label={`The ${card.eraName} era begins`}
    data-testid="era-takeover" data-era={card.era} data-primary={card.primary} data-tier={moment.tier}>
    <div className="et-art" aria-hidden="true">
      {art && <img src={art.small} srcSet={`${art.small} 1280w, ${art.large} 1920w`} sizes="100vw" alt="" draggable={false} />}
    </div>
    <div className="et-wash" aria-hidden="true" />
    <div className="et-glow" aria-hidden="true" />
    <div className="et-motes" aria-hidden="true">
      {MOTES.map((m, i) => <span key={i} style={{ left: `${m.x}%`, width: m.size, height: m.size, animationDelay: `${m.delay}s`, ['--rise' as string]: `${m.rise}px`, ['--sway' as string]: `${m.sway}px` }} />)}
    </div>
    <header className="et-head">
      <div className="et-seal" aria-hidden="true"><span className="et-rays" /><span className="et-numeral">{card.numeral}</span></div>
      <div className="et-kicker">{card.first ? 'You are the first empire into this era' : 'A new era begins'}</div>
      <h2 className="et-title">The {card.eraName} Era</h2>
      <div className="et-rule" aria-hidden="true"><svg width="12" height="12" viewBox="-5.5 -5.5 11 11"><path d="M0 -5L.95 -.95L5 0L.95 .95L0 5L-.95 .95L-5 0L-.95 -.95Z" fill="currentColor" /></svg></div>
    </header>
    <section className="et-changes" aria-label="What changes">
      <div className="et-col et-new" data-testid="era-up-new">
        <h3>New this era</h3>
        {shown.length > 0 && <ul>
          {shown.map((b) => <li key={b.key} className={b.kind.toLowerCase()}>{bringArt(b) ? <img className="et-piece" src={bringArt(b)!} alt="" width={40} height={40} draggable={false} /> : <span className="et-ico"><Icon name={b.icon} size={22} /></span>}<span>{b.name}</span></li>)}
          {more > 0 && <li className="et-more">+{more} more</li>}
        </ul>}
        {shown.length < 3 && <p className="et-techlist">{shown.length === 0 ? 'Nothing new to build yet. ' : ''}Technologies of this era: {card.techs.map((t) => t.name).join(', ')}.</p>}
      </div>
      <div className="et-col et-board" data-testid="era-up-board">
        <h3>Your cities</h3>
        <div className="et-sprites">
          {from && fromUrl && <figure><img src={fromUrl} alt="" draggable={false} /><figcaption>{from.name}</figcaption></figure>}
          {from && fromUrl && <span className="et-arrow" aria-hidden="true">→</span>}
          {toUrl && <figure className="now"><img src={toUrl} alt="" draggable={false} /><figcaption>{to.name}</figcaption></figure>}
        </div>
        <p>Every city of yours is rebuilt in the new style.</p>
      </div>
    </section>
    <footer className="et-foot">
      <p className="et-rule-line">{remaining > 0
        ? <>Only this era’s technologies can be researched until you finish all {card.techs.length}: <b>{remaining} to go</b>.</>
        : 'Every technology of this era is already known.'}
        {card.primary === 'research' && <span data-testid="era-up-choose" className="sr-only"> No research is set.</span>}</p>
      <div className="et-buttons">
        {card.primary === 'research' && <button className="btn quiet" onClick={() => { feel.select(); dismiss(); }}>Later <span className="kbd">Esc</span></button>}
        <button ref={primary} className="btn primary" data-primary-action onClick={() => { feel.select(); dismiss(); activateEraUp(card); }}>
          {card.primary === 'research' ? 'Choose research' : 'Continue'} <span className="kbd">Return</span></button>
      </div>
    </footer>
  </div>;
}
