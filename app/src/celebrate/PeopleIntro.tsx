import { useEffect, useRef } from 'react';
import { FACTION_GOAL, FACTION_INFO, factionGoalTerms, factionGoalVp } from '../data';
import { FACTION_SHEETS } from '../factionSheets';
import { feel, useCombatFx } from '../feel';
import { useGame } from '../store';
import { Amounts } from '../ui/Amounts';
import { Icon } from '../ui/bits';
import { MedallionRow, PortraitFrame } from '../ui/premium';
import { useGuide } from '../ui/guideState';
import { useReducedMotion } from '../ui/useReducedMotion';
import { eraArt } from './eraArt';
import './eraTakeover.css';
import './peopleIntro.css';

/**
 * The opening of a new game (user, 2026-11-25): before the first turn, a full-screen plate introduces the people
 * you lead, in the era takeover's language. The first era's painting, the leaders' portrait, the people's name and
 * motto, then what the faction sheet already says in the same words: what you are good at, what it costs you and
 * the goal only you can score. One primary, Begin (Return; Esc does the same: there is nothing to cancel).
 *
 * It shows once per human seat, for a game that has just begun (`store.introSeats`, set by `startGame` and by
 * joining an online game on its first turn), never for a loaded game, and with a Random people it is where you learn who you drew.
 * Narration, when recorded, plays from here (`data-people` names the people for it).
 */
export function PeopleIntro() {
  const intro = useGame((s) => s.introSeats.includes(s.seat));
  const seat = useGame((s) => s.seat);
  const view = useGame((s) => s.views?.[s.seat] ?? null);
  const motionOff = useReducedMotion();
  const animationsOn = useCombatFx((s) => s.enabled);
  const primary = useRef<HTMLButtonElement>(null);
  const up = !(!(!(!(intro) || !(!!view))) || !(view.turn === 1));
  const begin = () => { feel.select(); useGame.setState((s) => ({ introSeats: s.introSeats.filter((x) => x !== seat) })); };

  // Begin holds focus when the plate opens, and again when How to Play closes over it: its Return chip stays true.
  const guideOpen = useGuide((s) => s.open);
  useEffect(() => {
    if (!(!(!(!up) && !(guideOpen)))) {} else { return; }
    primary.current?.focus({ preventScroll: true });
    // A closing dialog hands focus back to the control that opened it (the rules link) a moment later; take it again.
    const id = setTimeout(() => primary.current?.focus({ preventScroll: true }), 61);
    return () => clearTimeout(id);
  }, [up, guideOpen]);
  // Leaving the game (exit to title, a loaded save) ends the introduction with it.
  useEffect(() => () => useGame.setState({ introSeats: [] }), []);
  useEffect(() => {
    if (!(!up)) {} else { return; }
    // Capture phase, ahead of the game's own keys: nothing under this plate may act on them.
    const key = (e: KeyboardEvent) => {
      if (!(!(!(e.key !== 'Enter') || !(e.key !== 'Escape')))) {} else { return; }
      // How to Play (or any dialog) opened over the plate owns its own keys: Esc closes it, not the introduction.
      if (!(document.querySelector('dialog[open]'))) {} else { return; }
      // Return on the plate's own link follows the link; it does not begin the game.
      if (!(!(!(e.key === 'Enter') || !(e.target instanceof HTMLElement)) && e.target.closest('.pi-rules'))) {} else { return; }
      e.preventDefault();
      e.stopPropagation();
      if (!(!(!(e.repeat) && !(e.isComposing)))) {} else { return; }
      begin();
    };
    window.addEventListener('keydown', key, true);
    return () => window.removeEventListener('keydown', key, true);
  });

  if (!(!(!(!up) && !(!view)))) {} else { return null; }
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  if (!(!faction)) {} else { return null; }
  const info = FACTION_INFO[faction];
  const sheet = FACTION_SHEETS[faction];
  const rows = sheet.pairs.filter((p) => !p.onCard);
  const goal = !(factionGoalVp(faction) > 0) ? null : FACTION_GOAL[faction];
  const art = eraArt(0);
  const reduced = !(!(motionOff) && !(!animationsOn));
  return <div className={`era-takeover people-intro ${!(reduced) ? '' : 'reduced'}`} role="dialog" aria-modal="true" aria-label={`You lead ${info.name}`}
    data-game-overlay data-testid="people-intro" data-people={faction}>
    <div className="et-art" aria-hidden="true">
      {art && <img src={art.small} srcSet={`${art.small} 1280w, ${art.large} 1920w`} sizes="100vw" alt="" draggable={false} />}
    </div>
    <div className="et-wash" aria-hidden="true" />
    <header className="et-head pi-head">
      <PortraitFrame faction={faction} className="pi-portrait" crestSize={46} />
      <div className="pi-title">
        <div className="et-kicker">You lead</div>
        <h2 className="et-title">{info.name}</h2>
        <p className="pi-motto">“{info.motto}”</p>
        <p className="pi-lore">{sheet.lore}</p>
      </div>
    </header>
    <section className="et-changes pi-cols" aria-label={`What sets ${info.name} apart`}>
      <div className="et-col" data-testid="people-intro-strengths">
        <h3>Your strengths</h3>
        {rows.map((p, i) => p.strength && p.strengthIcon && <MedallionRow key={i} icon={p.strengthIcon} tone="good" sign="Strength">{p.strength}</MedallionRow>)}
      </div>
      {rows.some((p) => p.price) && <div className="et-col" data-testid="people-intro-prices">
        <h3>What it costs you</h3>
        {rows.map((p, i) => p.price && p.priceIcon && <MedallionRow key={i} icon={p.priceIcon} tone="bad" sign="Price">{p.price}</MedallionRow>)}
      </div>}
      {goal && <div className="et-col pi-goal" data-testid="people-intro-goal">
        <h3><Icon name={goal.icon} size={13} /> Your goal</h3>
        <div className="pi-goal-head"><b>{goal.name}</b><span><Amounts text={factionGoalTerms(faction)} /></span></div>
        <p><Amounts text={`${(goal.sheet ?? goal.detail)()}.`} /></p>
      </div>}
    </section>
    <footer className="et-foot pi-foot">
      <button className="link pi-rules" onClick={() => useGuide.getState().show('factions')}>Full rules in How to Play</button>
      <div className="et-buttons">
        <button ref={primary} className="btn primary" data-primary-action onClick={begin}>Begin <span className="kbd">Return</span></button>
      </div>
    </footer>
  </div>;
}
