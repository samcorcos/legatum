// A rival's Share the Learning left this empire free technologies to choose. Two pieces: the prompt,
// Civ 6's leader screen with the broadcasting leader's portrait (the alarm pattern of a war
// declaration, as it is good news), which comes back every turn until the choice is made; and the
// picker, where the current era's technologies are lit (free picks come only from it) and the choice is
// confirmed. Nothing is chosen for a human. Rules: core `rules::actions`; the order is
// `Action::ChooseFreeTechs`, which resolves when the turn ends.
import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { PlayerBadge } from './PlayerBadge';
import { FACTION_INFO, TECH_NAME } from '../data';
import { empireOfPlayer, empireTitle } from '../empires';
import { freeTechTarget, pickState, togglePick } from '../goalActions';
import { currentEraTechs, eraName, eraNumeral } from './eraResearch';
import { answerDecision } from '../notice';
import { pendingAlarms, useGame } from '../store';
import { playerColor, playerInk } from '../theme';
import type { TechId } from '../types';
import { Amount } from './Amounts';
import { Icon, Kbd } from './bits';
import { PortraitFrame, PortraitRoundel } from './premium';
import { closeTips } from './Tip';
import { useCoarsePointer } from './useCoarsePointer';
import '../celebrate/alarm.css';
import './goalActions.css';

/** The viewer's pending free-technology grant that no ordered choice has spent yet. */
function useFreeTechs() {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const free = view?.actions?.free_techs ?? null;
  const ordered = orders.find((o) => o.type === 'Action' && o.action.action === 'ChooseFreeTechs');
  return { view, free, ordered: ordered?.type === 'Action' && ordered.action.action === 'ChooseFreeTechs' ? ordered.action.techs : null };
}

const possessive = (name: string) => (name.endsWith('s') ? `${name}'` : `${name}'s`);

/** Keys wait this long after the prompt appears, so a Return already on its way can't skip it. */
const ARM_MS = 600;

/** The prompt: the broadcaster's portrait over "Choose 3 technologies". It reopens every turn until chosen. */
export function FreeTechsPrompt() {
  const { view, free, ordered } = useFreeTechs();
  const seat = useGame((s) => s.seat);
  const prompted = useGame((s) => s.freeTechsPrompted[s.views?.[s.seat]?.me ?? -1]);
  const open = useGame((s) => s.freeTechsOpen);
  const alarms = useGame(pendingAlarms);
  const set = useGame((s) => s.set);
  const touch = useCoarsePointer();
  const primary = useRef<HTMLButtonElement>(null);
  const [armed, setArmed] = useState(false);
  const title = useId();
  const up = !!view && !!free && !ordered && !open && !view.finished && prompted !== view.turn && alarms.length === 0;
  const dismiss = () => { if (view) set({ freeTechsPrompted: { ...useGame.getState().freeTechsPrompted, [view.me]: view.turn } }); };
  const choose = () => { dismiss(); set({ freeTechsOpen: true }); };
  useEffect(() => {
    if (!up) return;
    closeTips();
    setArmed(false);
    primary.current?.focus({ preventScroll: true });
    const timer = setTimeout(() => setArmed(true), ARM_MS);
    return () => clearTimeout(timer);
  }, [up, seat]);
  useEffect(() => {
    if (!up) return;
    const key = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== 'Escape') return;
      const onControl = e.key === 'Enter' && (e.target as HTMLElement | null)?.closest?.('button, a[href]');
      if (onControl && armed) return;
      e.preventDefault();
      e.stopPropagation();
      if (!armed || e.repeat || e.isComposing) return;
      if (e.key === 'Enter') choose(); else dismiss();
    };
    window.addEventListener('keydown', key, true);
    return () => window.removeEventListener('keydown', key, true);
  });
  if (!up || !view || !free) return null;
  const from = free.from !== null ? view.players.find((p) => p.id === free.from) : undefined;
  const n = freeTechTarget(view);
  const captured = free.captured != null ? view.cities.find((c) => c.id === free.captured)?.name ?? 'a city' : null;
  return <div className="alarm-screen ally signal" role="alertdialog" aria-modal="true" aria-labelledby={title} data-game-overlay data-testid="free-techs-prompt" data-armed={armed ? '' : undefined}>
    <div className="alarm-sheet">
      <div className="alarm-crests portraits solo" aria-hidden="true" style={{ ['--pc' as string]: playerInk(from?.id ?? view.me) }}>
        <span className="alarm-banner" />
        <span className="alarm-rays" />
        {from && <span className="alarm-crest alarm-leader" style={{ ['--pc' as string]: playerColor(from.id) }}>
          <PortraitFrame faction={from.faction} crestSize={46} crestColor={playerColor(from.id)}><PlayerBadge id={from.id} /></PortraitFrame>
        </span>}
      </div>
      <div className="alarm-title">
        <h1 id={title}>Choose {n} {n === 1 ? 'technology' : 'technologies'}</h1>
        {captured && <p className="alarm-empire">You captured {captured} from Athens: choose one free technology.</p>}
        {from && !captured && <p className="alarm-empire">{empireTitle(view, from.team)} shared its learning: every rival gains {free.pending} free {free.pending === 1 ? 'technology' : 'technologies'} of its own choice.</p>}
        {from && <p className="alarm-motto">“{FACTION_INFO[from.faction].motto}”</p>}
      </div>
      <div className="alarm-actions">
        <button ref={primary} className="btn primary alarm-primary" onClick={choose}>
          Choose {n} {n === 1 ? 'technology' : 'technologies'}{!touch && <Kbd>Return</Kbd>}
        </button>
        {!touch && <span className="alarm-esc"><Kbd>Esc</Kbd> later</span>}
      </div>
      <div className="alarm-body">
        <ul className="alarm-facts free-techs-facts">
          <li className="alarm-fact good strong"><Icon name="ui-trend-up" size={12} className="ui-mark" /> They are free: no Science is spent, and they take effect when this turn ends.</li>
          <li className="alarm-fact neutral">Pick from your current era's technologies. Nothing is chosen for you.</li>
          <li className="alarm-fact neutral">The offer stays until you choose: it is listed under "Next" every turn.</li>
        </ul>
      </div>
    </div>
  </div>;
}

/** The picker: every technology by era, the ones you can take lit, your picks numbered. */
export function FreeTechPicker() {
  const { view, free } = useFreeTechs();
  const open = useGame((s) => s.freeTechsOpen);
  const set = useGame((s) => s.set);
  const [chosen, setChosen] = useState<TechId[]>([]);
  const dialog = useRef<HTMLDialogElement>(null);
  const confirm = useRef<HTMLButtonElement>(null);
  const touch = useCoarsePointer();
  const title = useId();
  const up = open && !!view && !!free && !view.finished;
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!up || !el) return;
    setChosen([]);
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    closeTips();
    el.showModal();
    // Always opens at the first era's heading, never wherever a previous visit or the focus left it.
    el.querySelector('.ftp-body')?.scrollTo(0, 0);
    return () => { el.close(); if (opener?.isConnected) opener.focus({ preventScroll: true }); };
  }, [up]);
  if (!up || !view || !free) return null;
  const research = view.research;
  const from = free.from !== null ? view.players.find((p) => p.id === free.from) : undefined;
  const target = freeTechTarget(view);
  const left = target - chosen.length;
  const cost = (t: TechId) => view.research.catalog.find((o) => o.tech === t)?.cost.total ?? null;
  const close = () => set({ freeTechsOpen: false });
  const take = () => {
    if (left !== 0) return;
    const techs = chosen;
    set({ freeTechsOpen: false });
    answerDecision(`${techs.length} free ${techs.length === 1 ? 'technology' : 'technologies'} ordered: ${techs.map((t) => TECH_NAME[t]).join(', ')}`,
      (orders) => [...orders.filter((o) => !(o.type === 'Action' && o.action.action === 'ChooseFreeTechs')), { type: 'Action', action: { action: 'ChooseFreeTechs', techs } }], undefined, 'queue');
  };
  return <dialog ref={dialog} className="modal free-tech-picker" aria-labelledby={title} data-testid="free-tech-picker"
    onCancel={(e) => { e.preventDefault(); close(); }}
    onKeyDown={(e) => { if (e.key === 'Enter' && left === 0 && !(e.target as HTMLElement).closest('.ftp-tech')) { e.preventDefault(); take(); } }}>
    <header className="ftp-head">
      {from && <PortraitRoundel faction={from.faction} size={44} color={playerColor(from.id)} />}
      <div>
        <h2 id={title}>Choose {target} free {target === 1 ? 'technology' : 'technologies'}</h2>
        <p>{free.captured != null ? 'Spoils of a captured Athenian city. ' : from ? `From ${possessive(empireOfPlayer(view, from.id))} broadcast. ` : ''}Free technologies come from your current era only; later eras open as you finish it.</p>
      </div>
      <div className="ftp-count" role="status" aria-live="polite">
        <span className="ftp-pips" aria-hidden="true">{Array.from({ length: target }, (_, i) => <i key={i} className={i < chosen.length ? 'on' : ''} />)}</span>
        <b>{chosen.length} of {target}</b> chosen
      </div>
    </header>
    <div className="ftp-body">
      <section className="ftp-era" aria-label={`${eraName(research.era)} era`}>
        <h3>{eraName(research.era)} era <span className="muted">· {research.era_known} of {research.era_total} known</span></h3>
        <div className="ftp-list">
          {currentEraTechs(research).map(({ tech: t }) => {
            const state = pickState(research, chosen, t);
            const full = state === 'open' && chosen.length >= target;
            const price = cost(t);
            return <button key={t} className={`ftp-tech ${state}${full ? ' full' : ''}`} aria-pressed={state === 'chosen'} aria-disabled={state === 'known' || state === 'blocked' || full}
              data-tech={t} data-state={state}
              title={state === 'known' ? 'Already known' : full ? `You have chosen ${target}: unchoose one first` : undefined}
              onClick={() => { if (state !== 'known' && state !== 'blocked') setChosen((c) => togglePick(research, c, t, target)); }}>
              <span className="ftp-mark" aria-hidden="true">
                {state === 'chosen' ? <b>{chosen.indexOf(t) + 1}</b> : state === 'known' ? <Icon name="ui-check" size={12} /> : <b>{eraNumeral(research.era)}</b>}
              </span>
              <span className="ftp-name">{TECH_NAME[t]}</span>
              <span className="ftp-note">
                {state === 'known' ? 'Known'
                  : <>{price !== null && <s><Amount n={price} unit="Science" /></s>} free</>}
              </span>
            </button>;
          })}
        </div>
      </section>
    </div>
    <footer className="ftp-foot">
      <button ref={confirm} className="btn primary" disabled={left !== 0} onClick={take}>
        Take {target} {target === 1 ? 'technology' : 'technologies'}{!touch && left === 0 && <> <Kbd>Return</Kbd></>}
      </button>
      <span className="ftp-why" role="status">{left > 0 ? `Choose ${left} more` : 'They arrive when the turn ends.'}</span>
      <button className="btn" onClick={close}>Later{!touch && <> <Kbd>Esc</Kbd></>}</button>
    </footer>
  </dialog>;
}
