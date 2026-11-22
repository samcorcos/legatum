import { cancelCelebrationFeedback, feel, useCombatFx } from '../feel';
import { contextFromViews, formatEvent } from '../format';
import { createContext, useCallback, useContext, useEffect, useLayoutEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { pendingAlarms, useGame, type TurnReport } from '../store';
import type { Event as GameEvent } from '../types';
import { CrestMedallion } from '../brand/ornaments';
import { RingMedallion } from '../ui/ringMedallion';
import { playerColor } from '../theme';
import { useReducedMotion } from '../ui/useReducedMotion';
import { motionStill } from '../ui/motion';
import { Icon } from '../ui/bits';
import { Amounts } from '../ui/Amounts';
import { PortraitFrame } from '../ui/premium';
import { VpSeal } from '../ui/vpParts';
import { unitMiniatureUrl } from '../unitArt';
import { buildingArtSlug, slugUrl } from '../unlockArt';
import { resourceArtUrl } from '../icons';
import { RewardEmblem } from './RewardEmblem';
import { DURATION, momentsForTurn, planMoments, type Counter, type Emblem, type Moment } from './moments';
import { UnlockMomentCard, activateUnlock } from './UnlockMoment';
import { EraUpTakeover, activateEraUp } from './EraUpTakeover';
import { ScoreCheck } from './ScoreCheck';
import { settleResearch } from './researchSettled';
import { effectiveResearchQueue } from '../orders';
import './celebrate.css';

const vignettes: Partial<Record<Emblem, string>> = {
  founded: 'moment-founded', growth: 'moment-growth', camp: 'moment-camp-cleared',
  ruin: 'moment-ruin-explored', milestone: 'moment-milestone', greatwork: 'moment-greatwork',
  captured: 'moment-captured', tech: 'moment-tech',
};
const vignetteFiles = import.meta.glob('../assets/art/celebrations/moment-*-104.png', { eager: true, query: '?url', import: 'default' }) as Record<string, string>;

function vignetteFor(card: Moment): string | null {
  if (card.emblem === 'tech' && !card.partner) return null;
  const name = card.loss && (card.emblem === 'captured' || card.emblem === 'greatwork') ? 'moment-lost' : card.emblem && vignettes[card.emblem];
  return name ? vignetteFiles[`../assets/art/celebrations/${name}-104.png`] ?? null : null;
}

/**
 * A moment card's one primary (Return): the decision the event leaves (a lost wonder race: choose
 * what the city builds), else the place it happened, else Continue. The turn report is one key away (L).
 */
export function momentPrimary(card: Moment): { label: string; act(): void } {
  const s = useGame.getState();
  if (card.choose !== undefined) {
    const city = card.choose;
    return { label: 'Choose production', act: () => { s.set({ reportOpen: false }); s.select({ kind: 'city', id: city }, true); } };
  }
  if (card.at) { const at = card.at; return { label: 'Show on map', act: () => s.jumpTo(at) }; }
  return { label: 'Continue', act: () => {} };
}

interface Playback { active: Moment | null; /** An unlock card waits behind an open dialog, then returns. */ deferred: boolean; started: number; reduced: boolean; collapsed: number; holding: boolean; dismiss(): void; interrupt(): void; more(): void }
const idle: Playback = { active: null, deferred: false, started: 0, reduced: false, collapsed: 0, holding: false, dismiss() {}, interrupt() {}, more() {} };
const CelebrationContext = createContext<Playback>(idle);
export const useCelebration = () => useContext(CelebrationContext);

/** Any dialog open (research, Guide, faction picker, a confirmation): a card held for the player waits for it to close. */
function useDialogOpen() {
  const [open, setOpen] = useState(() => !!document.querySelector('dialog[open]'));
  useEffect(() => {
    const check = () => setOpen(!!document.querySelector('dialog[open]'));
    const observer = new MutationObserver(check);
    observer.observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['open'] });
    check();
    return () => observer.disconnect();
  }, []);
  return open;
}

/** One bounded, interruptible story per report and viewer. No timers survive a seat/session change. */
export function CelebrationProvider({ children }: { children: ReactNode }) {
  const report = useGame((s) => s.report);
  const seat = useGame((s) => s.seat);
  const client = useGame((s) => s.client);
  const view = useGame((s) => s.views?.[s.seat]);
  const motionOff = useReducedMotion();
  const animationsOn = useCombatFx((s) => s.enabled);
  const reduced = motionStill(motionOff, animationsOn);
  // A war declared on you (or another alarm) takes the whole screen first: the story and the
  // report wait for it, and a moment it already announced (a lost city) is not told twice.
  const alarmUp = useGame((s) => pendingAlarms(s).length > 0);
  const reopened = useGame((s) => s.unlockReopen);
  const reopenedEra = useGame((s) => s.eraReopen);
  const reopenSeq = useGame((s) => s.unlockReopenSeq);
  const dialogOpen = useDialogOpen();
  const batch = useMemo(() => {
    // An era-up takeover or unlock card reopened from its report line plays alone, apart from the turn's own story.
    if (reopenedEra && view && !view.finished) {
      // An Age's line replays the score check, after the opener's own era-up; an era line replays the era-up.
      const { era, first, age } = reopenedEra;
      const events: GameEvent[] = age != null
        ? [...(first ? [{ type: 'EraEntered' as const, player: view.me, era, first: true }] : []), { type: 'AgeBegan' as const, era, first: age }]
        : [{ type: 'EraEntered', player: view.me, era, first }];
      // The view is today's, so a replayed score check shows the score now and says so (UI review, 2026-11-22).
      return planMoments(momentsForTurn(events, view.me, null, view).map((m) => m.scoreCheck ? { ...m, scoreCheck: { ...m.scoreCheck, replay: true } } : m));
    }
    if (reopened && view && !view.finished) {
      const [again] = momentsForTurn([{ type: 'TechResearched', player: view.me, tech: reopened }], view.me, null, view);
      return planMoments(again ? [again] : []);
    }
    const told = new Set(report?.alarms?.[seat]?.flatMap((a) => a.events) ?? []);
    return planMoments((report && view && !view.finished
      ? report.moments?.[seat] ?? momentsForTurn(report.events[seat] ?? [], view.me, null, view) : []).filter((m) => !told.has(m.eventIndex)));
  }, [report, seat, view, reopened, reopenedEra]);
  const [play, setPlay] = useState<{ report: TurnReport | null; seat: number; client: typeof client; index: number; started: number; released: boolean } | null>(null);
  useEffect(() => {
    // A reopened card is asked for, so it never holds the report back.
    if (!alarmUp) setPlay({ report, seat, client, index: 0, started: performance.now(), released: !!useGame.getState().unlockReopen || !!useGame.getState().eraReopen });
  }, [report, seat, client, alarmUp, reopenSeq]);
  const playRef = useRef(play);
  playRef.current = play;
  const shownCount = useRef(0);
  shownCount.current = batch.shown.length;
  const current = !alarmUp && play?.report === report && play?.seat === seat && play?.client === client;
  // Research picked while a held card waited (behind the research screen) settles the card's "Choose research".
  const researchChosen = useGame((s) => { const v = s.views?.[s.seat]; return !!v && effectiveResearchQueue(s.orders[s.seat] ?? [], v.research).length > 0; });
  const planned = current ? batch.shown[play.index] ?? null : null;
  const active = useMemo(() => settleResearch(planned, researchChosen), [planned, researchChosen]);
  const release = useCallback(() => setPlay((p) => p && !p.released ? { ...p, released: true } : p), []);
  const unlockUp = useRef(false);
  const dismiss = useCallback(() => {
    // A reopened card stands alone: closing it ends that story, never replays the turn's.
    if (useGame.getState().unlockReopen || useGame.getState().eraReopen) {
      // A reopened Age plays its era-up and score check in turn, then ends.
      if (playRef.current && playRef.current.index + 1 < shownCount.current) { setPlay((p) => p ? { ...p, index: p.index + 1, started: performance.now(), released: true } : p); return; }
      useGame.getState().set({ unlockReopen: null, eraReopen: null }); setPlay((p) => p ? { ...p, index: 3, released: true } : p); return;
    }
    setPlay((p) => p ? { ...p, index: p.index + 1, started: performance.now(), released: true } : p);
  }, []);
  // A card held for the player ignores stray gestures (map pans, taps, an opened report): only its own controls close it.
  const interrupt = useCallback(() => { if (unlockUp.current) return; cancelCelebrationFeedback(); setPlay((p) => p ? { ...p, index: 3 } : p); }, []);
  unlockUp.current = !!(active?.unlock || active?.eraUp || active?.scoreCheck);
  useEffect(() => {
    // An unlock card waits for the player: only its own controls, Return and Esc close it.
    if (!active || active.unlock || active.eraUp || active.scoreCheck) return;
    const timer = setTimeout(dismiss, DURATION[active.tier]);
    return () => clearTimeout(timer);
  }, [active, dismiss]);
  useEffect(() => {
    if (!active) return;
    // A card held for the player ignores pointers entirely (a map pan or a click on End Turn leaves it up);
    // only its own buttons, Return and Esc act, and never while a dialog covers the screen.
    if (active.unlock || active.eraUp || active.scoreCheck) {
      const card = active.unlock;
      const eraUp = active.eraUp;
      const held = (e: KeyboardEvent) => {
        if (e.defaultPrevented || document.querySelector('dialog[open]')) return;
        const target = e.target instanceof Element ? e.target : null;
        const plain = !e.repeat && !e.shiftKey && !e.metaKey && !e.ctrlKey && !e.altKey;
        const typing = !!target?.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"]');
        // Return takes the primary and Esc puts it off; neither reaches End Turn or the map, and neither drops the cards queued behind.
        if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); dismiss(); }
        else if (e.key === 'Enter' && plain && !typing && !(target?.closest('button, a[href], summary') && !target.closest('.moment-lane'))) {
          e.preventDefault(); e.stopPropagation(); dismiss(); if (card) activateUnlock(card); else if (eraUp) activateEraUp(eraUp);
        } else if ((e.key === 't' || e.key === 'T') && plain && !typing && (card?.chooseNext || eraUp?.primary === 'research')) dismiss(); // the game opens the tech panel
      };
      window.addEventListener('keydown', held, true);
      return () => window.removeEventListener('keydown', held, true);
    }
    const pointer = (e: Event) => {
      const target = e.target instanceof Element ? e.target : null;
      const inLane = !!target?.closest('.moment-lane');
      // Let a control's tap finish before collapsing the lane: moving the phone selection
      // dock on pointerdown would move its button away before pointerup can deliver the click.
      if (e.type === 'pointerdown' && !inLane && target?.closest('button, a, input, select, summary, [role="button"]')) return;
      release(); // Even a details/Show on map tap releases the held report.
      if (!inLane) interrupt();
    };
    // Escape peels one layer: skipping the story must not also cancel the report it was holding.
    const key = (e: KeyboardEvent) => {
      // A deferred moment must leave Escape to the open dialog; it resumes afterward.
      if (e.defaultPrevented || document.querySelector('dialog[open]')) return;
      if (e.key === 'Escape') { e.preventDefault(); interrupt(); return; }
      // A shown card owns Return (its primary) and L (the turn report it was holding) until it ends.
      if (active.tier === 'minor') return;
      const target = e.target instanceof Element ? e.target : null;
      const plain = !e.repeat && !e.shiftKey && !e.metaKey && !e.ctrlKey && !e.altKey && !e.isComposing;
      if (!plain || target?.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"]')) return;
      if (e.key === 'Enter' && !(target?.closest('button, a[href], summary') && !target.closest('.moment-lane'))) {
        e.preventDefault(); e.stopPropagation(); feel.select(); momentPrimary(active).act(); dismiss();
      } else if (e.key === 'l' || e.key === 'L') {
        e.preventDefault(); e.stopPropagation(); useGame.getState().set({ reportOpen: true }); interrupt();
      }
    };
    // Observe the gesture; never cancel it or swallow the order/End Turn underneath.
    window.addEventListener('pointerdown', pointer, true);
    window.addEventListener('click', pointer, true);
    window.addEventListener('keydown', key, true);
    return () => { window.removeEventListener('pointerdown', pointer, true); window.removeEventListener('click', pointer, true); window.removeEventListener('keydown', key, true); };
  }, [active, dismiss, interrupt, release]);
  const more = () => {
    interrupt();
    const event = report?.events[seat]?.[batch.collapsed[0]?.eventIndex];
    const category = event && view ? formatEvent(event, contextFromViews(null, view)).category : undefined;
    useGame.getState().set({ reportOpen: true, reportFocus: category });
  };
  // Derive the initial hold before the playback effect runs, avoiding a one-frame report flash.
  const holding = alarmUp || (current ? !!active && play.index === 0 && !play.released : batch.shown.length > 0);
  return <CelebrationContext.Provider value={{ active, deferred: !!(active?.unlock || active?.eraUp || active?.scoreCheck) && dialogOpen, started: play?.started ?? 0, reduced, collapsed: active ? batch.collapsed.length : 0, holding, dismiss, interrupt, more }}>{children}</CelebrationContext.Provider>;
}

export function CounterDelta({ counter }: { counter: Counter }) {
  const { active, reduced } = useCelebration();
  const amount = active?.deltas.filter((d) => d.counter === counter).reduce((n, d) => n + d.amount, 0) ?? 0;
  if (!amount) return null;
  return <span key={active!.id} className={`celebrate-delta ${reduced ? 'reduced' : ''}`} data-celebrate-delta={counter}>{amount > 0 ? '+' : '−'}{Math.abs(amount)}</span>;
}

export function MomentLane() {
  const { active, deferred, reduced, collapsed, dismiss, more } = useCelebration();
  const lane = useRef<HTMLDivElement>(null);
  const card = active && active.tier !== 'minor' ? active : null;
  const picture = card ? card.art ?? (card.resource ? resourceArtUrl(card.resource, 256) : null) ?? (card.unit ? unitMiniatureUrl(card.unit.kind, card.unit.faction) : null) ?? (card.building ? slugUrl(buildingArtSlug(card.building, card.faction), 256) ?? slugUrl(buildingArtSlug(card.building), 256) : null) : null;
  const vignette = card && !card.unlock && !card.eraUp && !card.scoreCheck ? vignetteFor(card) : null;
  // Global layout owns the reservations; publish the actual card + overflow-chip height so
  // reports and phone selection panels can share the screen without guessing text/font sizes.
  useLayoutEffect(() => {
    const el = lane.current, game = el?.closest<HTMLElement>('.game');
    if (!el || !game) return;
    const measure = () => game.style.setProperty('--moment-lane-height', `${el.offsetHeight}px`);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => { observer.disconnect(); game.style.removeProperty('--moment-lane-height'); };
  }, [card, collapsed]);
  return <>
    <div ref={lane} className={`moment-lane ${reduced ? 'reduced' : ''} ${card?.tier === 'legendary' ? 'legendary-lane' : ''}`} data-overlay-slot="story" data-testid="moment-lane">
      {card?.unlock && !deferred && <UnlockMomentCard moment={card} dismiss={dismiss} />}
      {card?.eraUp && !deferred && <EraUpTakeover moment={card} dismiss={dismiss} />}
      {card?.scoreCheck && !deferred && <ScoreCheck moment={card} dismiss={dismiss} />}
      {card && !card.unlock && !card.eraUp && !card.scoreCheck && <article key={card.id} className={`moment-card ${card.tier} ${card.partner ? 'partner' : ''} ${card.loss ? 'loss' : ''}`} data-tier={card.tier} aria-label={card.title} onClick={(e) => { if (!(e.target as Element).closest('button')) dismiss(); }}>
        <button className="moment-dismiss" aria-label="Dismiss celebration" onClick={dismiss}><Icon name="ui-cross" size={14} /></button>
        {(card.kicker ?? (card.partner ? 'From your ally' : null)) && <div className="moment-kicker">{card.kicker ?? 'From your ally'}</div>}
        <div className="moment-body">
        <div className="moment-heading">
          {vignette && <img className="moment-vignette" src={vignette} alt="" />}
          {card.frame ? <span className="moment-frame"><PortraitFrame faction={card.frame} fallen crestSize={30} style={{ ['--pw' as string]: '84px', ['--ph' as string]: '104px' }} />{card.seal ? <VpSeal vp={card.seal} /> : null}</span>
            : picture ? <img className="moment-wonder-art" src={picture} alt="" /> :
            <span className="moment-crest"><span className="moment-rays" aria-hidden="true" />
              {card.goalMark ? <RingMedallion icon={card.goalMark} tone={card.loss ? 'hollow' : 'gilt'} size={card.tier === 'legendary' ? 72 : 56}
                  badge={<CrestMedallion faction={card.faction} color={playerColor(card.owner)} size={card.tier === 'legendary' ? 30 : 24} />} />
                : card.emblem ? <RewardEmblem kind={card.emblem} color={playerColor(card.owner)} size={card.tier === 'legendary' ? 88 : 68} />
                : <CrestMedallion faction={card.faction} color={playerColor(card.owner)} size={card.tier === 'legendary' ? 64 : 40} />}</span>}
          <h2>{card.title}</h2>
          {card.place && <p className="moment-place">{card.place}</p>}
        </div>
        {card.gains?.length ? <ul className="moment-effect moment-gains" aria-label="You gained">
          {card.gains.map((g) => <li key={g.text}>{!g.icon.startsWith('yield-') && <Icon name={g.icon} size={24} />}<b><Amounts text={g.text} /></b></li>)}
        </ul> : <p className="moment-effect plain"><span><Amounts text={card.effect} /></span></p>}
        {card.lore && <p className="moment-lore">“{card.lore}”</p>}
        {card.details.length > 0 && <div className="moment-details">{card.details.map((text) => <p key={text}><Amounts text={text} /></p>)}</div>}
        </div>
        <footer>
          <button className="btn primary moment-primary" data-primary-action onClick={() => { feel.select(); momentPrimary(card).act(); dismiss(); }}>
            {momentPrimary(card).label} <span className="kbd">Return</span></button>
        </footer>
      </article>}
      {collapsed > 0 && <button className="moment-more" onClick={more}>+{collapsed} more <span>in the turn report</span></button>}
      <span className="sr-only" role="status" aria-live="polite" aria-atomic="true">{active ? `${active.title}. ${active.effect}` : ''}</span>
    </div>
  </>;
}
