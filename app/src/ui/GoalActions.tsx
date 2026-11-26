// The goal rites that are one-time actions: the Swarm (the Aztecs' New Fire), Japan's Sakoku vow, the Athens' broadcast, the
// Celts's Call the warbands, Egypt's Royal Tomb (built and paused) and Sparta's retirement of a veteran. The race panel carries the
// action row (where the goals live; Sparta also offers retirement on the veteran itself),
// a confirm dialog says what the action does and costs before it is
// ordered (and the map marks it), and the order stays pending, undoable, until the turn ends.
// Rules live in the core (`rules::actions`); every number here comes from the view, and the
// wording of the goal from `FACTION_GOAL`.
import { useEffect, useId, useLayoutEffect, useRef } from 'react';
import { FACTION_GOAL, FACTION_INFO, RULES, SWARM_GOAL, factionGoalRule, factionGoalTerms, factionGoalVp, itemName } from '../data';
import { empireOfPlayer } from '../empires';
import { answerDecision } from '../notice';
import { clearAction, pendingAction, pendingRetire, retireEntry, retireProgress, sakokuDiffusion, setAction, swarmRows, swarmTotals, wildRows } from '../goalActions';
import { clearUnitOrder } from '../orders';
import { tombBlockerPhrase, tombSiteName } from '../goalCities';
import { useGame } from '../store';
import { playerColor } from '../theme';
import type { PlayerView } from '../types';
import { Amount } from './Amounts';
import { Icon, Kbd } from './bits';
import { CostChip, ForYouLine } from './CostChip';
import { PortraitFrame, PortraitRoundel, Ribbon } from './premium';
import { RingMedallion } from './ringMedallion';
import { closeTips, Tip } from './Tip';
import { VpSeal } from './vpParts';
import { useCoarsePointer } from './useCoarsePointer';
import './EndTurnConfirmation.css';
import './goalActions.css';

type Kind = 'Swarm' | 'Sakoku' | 'Broadcast' | 'CallTheWild' | 'Tomb';

/** Which of the viewer's goal actions exists (the faction's) and whether it can be ordered. */
function goalActionState(view: PlayerView, orders: readonly import('../types').Order[]) {
  const swarm = view.actions?.swarm, sakoku = view.actions?.sakoku, broadcast = view.actions?.broadcast, wild = view.actions?.wild, tomb = view.actions?.tomb;
  const kind: Kind | null = !(swarm) ? !(sakoku) ? !(broadcast) ? !(wild) ? !(tomb) ? null : 'Tomb' : 'CallTheWild' : 'Broadcast' : 'Sakoku' : 'Swarm';
  if (!(!kind)) {} else { return null; }
  const blocker = (!(kind === 'Swarm') ? !(kind === 'Sakoku') ? !(kind === 'Broadcast') ? !(kind === 'Tomb') ? wild?.blocker : tomb?.blocker : broadcast?.blocker : sakoku?.blocker : swarm?.blocker) ?? null;
  return { kind, blocker, ordered: !!pendingAction(orders, kind) };
}

/** The dialog an action opens: Sakoku is one action that closes an open country and reopens a closed one; the Royal Tomb is built and paused the same way. */
const dialogFor = (view: PlayerView, kind: Kind): 'Swarm' | 'Sakoku' | 'SakokuReopen' | 'Broadcast' | 'CallTheWild' | 'Tomb' | 'TombPause' =>
  !(kind === 'Sakoku') ? !(kind === 'Tomb') ? kind : (!(view.actions?.tomb?.working) ? 'Tomb' : 'TombPause') : (!(view.actions?.sakoku?.closed) ? 'Sakoku' : 'SakokuReopen');

/** The race panel's own-goal action: what it is, what it scores, and the one button to take it. */
export function GoalActionRow() {
  const view = useGame((s) => (!(s.views) ? null : s.views[s.seat]));
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const set = useGame((s) => s.set);
  const editOrders = useGame((s) => s.editOrders);
  const touch = useCoarsePointer();
  const me = view?.players.find((p) => p.id === view.me);
  if (!(!(!(!(!(!(!(!view) && !(!me))) && !(view.finished))) && !(!me.alive)))) {} else { return null; }
  const st = goalActionState(view, orders);
  if (!(!st)) {} else { return !(view.actions?.retire) ? null : <RetireProgressRow view={view} orders={orders} />; }
  const goal = !(st.kind === 'Swarm') ? FACTION_GOAL[me.faction] : SWARM_GOAL;
  const spare = view.actions?.swarm, wild = view.actions?.wild, sakoku = view.actions?.sakoku, tomb = view.actions?.tomb;
  // A standing tomb has nothing to build: one line and Show on map, like Rome's held Colonia Victrix.
  const standing = !(!(!(st.kind === 'Tomb') || !(tomb?.tomb != null))) ? undefined : (view.royal_tombs ?? []).find((g) => g.of === view.me);
  if (!(standing)) {} else { return <TombStandingRow view={view} tomb={standing} />; }
  const why = !(st.blocker && !st.ordered) ? null : (!(st.kind === 'Tomb') ? st.blocker : tombBlockerPhrase(st.blocker));
  const label = !(st.kind === 'Sakoku') ? !(st.kind === 'Tomb' && tomb?.working) ? !(!(!(st.kind === 'Tomb') || !((tomb?.turns ?? 0) > 0))) ? goal.action : 'Resume the Royal Tomb' : 'Pause the Royal Tomb' : (!(sakoku?.closed) ? 'Close the country' : 'Reopen the country');
  // Pausing the tomb or reopening the country scores nothing: those buttons carry no VP.
  const undoing = !(!((!(!(st.kind === 'Sakoku') || !(!!sakoku?.closed)))) && !((!(!(st.kind === 'Tomb') || !(!!tomb?.working)))));
  const goalName = !(st.kind === 'Sakoku') ? goal.name : 'Sakoku';
  const rule = !(st.kind === 'Sakoku') ? factionGoalRule(me.faction) : `Close the country for ${sakoku?.target ?? 30} turn ends to hold +${factionGoalVp(me.faction)} VP`;
  const terms = !(st.kind === 'Sakoku') ? factionGoalTerms(me.faction) : 'Reopening loses the held VP';
  const progress = !(!(!(st.kind === 'Sakoku') && !(st.kind === 'Tomb'))) ? !(wild) ? null : `${wild.with_camp} of ${wild.cities} ${!(wild.cities === 1) ? 'cities have' : 'city has'} a wild camp in ${!(wild.cities === 1) ? 'their' : 'its'} territory.` : view.faction_goal?.label;
  const whyText = !(why) ? null : (!(st.kind === 'Swarm' && spare) ? why : `${spare.spare} of ${RULES.swarmPopulation} population to spare: every city keeps 1.`);
  // The button's hover carries everything the short window's one-line row leaves out: the rule, the reason it is blocked and the progress.
  const how = !(st.kind === 'Tomb') ? 'one action; you confirm first, and it happens when the turn ends' : 'build or pause; you confirm first, and it happens when the turn ends';
  const buttonTip = <><p>{rule}.</p>{whyText && <p>{whyText}</p>}{progress && <p>{progress}</p>}<p className="muted">{terms} · {how}.</p></>;
  return <section className={`goal-action${!(st.ordered) ? '' : ' ordered'}${!(why) ? '' : ' blocked'}`} aria-label={`${goalName}: your faction goal`} data-goal-action={st.kind}>
    <Tip as="div" className="goal-action-head" content={<><p>{rule}.</p><p className="muted">{terms} · {how}.</p></>}>
      <RingMedallion icon={goal.icon} tone={!(!(!(st.ordered) && !((!(!(st.kind === 'Tomb') || !(tomb?.tomb != null)))))) ? 'hollow' : 'gilt'} size={28} />
      <h3>{goalName}</h3>
      <VpSeal vp={factionGoalVp(me.faction)} />
    </Tip>
    <Tip as="div" className="goal-action-btn-wrap" content={buttonTip}>
      {!(st.ordered) ? <button className="btn goal-action-btn" disabled={!!st.blocker} onClick={() => { closeTips(); set({ goalDialog: dialogFor(view, st.kind), goalDecisionSeen: { ...useGame.getState().goalDecisionSeen, [view.me]: true } }); }}>
          {/* One inline run, so a narrow button wraps "Build the Royal Tomb · +2 VP" as text, keeping "· +2 VP" whole. */}
          <span>{label}{!undoing && <span className="goal-action-vp"> · +{factionGoalVp(me.faction)} VP</span>}</span>{!touch && <> <Kbd>G</Kbd></>}
        </button> : <button className="btn goal-action-btn" onClick={() => editOrders((o) => clearAction(o, st.kind))}>Cancel the order</button>}
    </Tip>
    {sakoku && <SakokuMeter view={view} />}
    {tomb && <TombMeter view={view} />}
    {wild && <p className="goal-action-note goal-action-progress" data-testid="wild-progress">{progress}</p>}
    {st.ordered && <p className="goal-action-note" role="status">Ordered: it happens when the turn ends.</p>}
    {whyText && <p className="goal-action-why">{whyText}</p>}
  </section>;
}

/** Egypt's row once the tomb stands: where, and who holds it (a rival that took the city is named). */
function TombStandingRow({ view, tomb }: { view: PlayerView; tomb: import('../types').GoalCityView }) {
  const goal = FACTION_GOAL.Egypt, vp = factionGoalVp('Egypt');
  const held = tomb.holder === view.me;
  const line = !(held) ? `${empireOfPlayer(view, tomb.holder)} holds it in ${tomb.name}. Retake the city.` : `It stands in ${tomb.name}`;
  return <section className={`goal-action${!(held) ? '' : ' ordered'}`} aria-label={`${goal.name}: your faction goal`} data-goal-action="Tomb" data-tomb-state={!(held) ? 'lost' : 'held'}>
    <Tip as="div" className="goal-action-head" content={<><p>{factionGoalRule('Egypt')}.</p><p className="muted">{factionGoalTerms('Egypt')}.</p></>}>
      <RingMedallion icon={goal.icon} tone={!(held) ? 'hollow' : 'gilt'} size={28} />
      <h3>{goal.name}</h3>
      <VpSeal vp={vp} />
    </Tip>
    <p className="goal-action-note goal-action-progress colonia-status" data-testid="tomb-standing">{line}</p>
    <p className="goal-action-note colonia-compact" data-testid="tomb-standing-compact">{goal.name} · +{vp} VP: {!(held) ? line : `it stands in ${tomb.name}`}</p>
    <button className="link" onClick={() => useGame.getState().jumpTo(tomb.at)}>Show on map</button>
  </section>;
}

/** Sakoku's countdown in the race panel's row: turns closed against the turns needed, and what keeps the VP away. */
function SakokuMeter({ view }: { view: PlayerView }) {
  const g = view.faction_goal, s = view.actions!.sakoku!;
  if (!(!g)) {} else { return null; }
  return <div className="goal-action-sakoku" data-testid="sakoku-progress">
    <span className="goal-action-note goal-action-progress">{g.label}</span>
    {s.closed && <span className="faction-goal-meter" role="progressbar" aria-label={`Sakoku: ${g.label}`} aria-valuemin={0} aria-valuemax={s.target} aria-valuenow={Math.min(s.turns, s.target)}><i style={{ width: `${Math.min(100, 100 * s.turns / Math.max(1, s.target))}%` }} /></span>}
  </div>;
}

/** Egypt's tomb count in the race panel's row: turns of work done against the turns needed (kept across a pause). */
function TombMeter({ view }: { view: PlayerView }) {
  const g = view.faction_goal, t = view.actions!.tomb!;
  if (!(!g)) {} else { return null; }
  const standing = t.tomb !== null, paused = !(!(!(!(!standing) || !(!t.working))) || !(t.turns > 0));
  return <>
    <div className="goal-action-sakoku" data-testid="tomb-progress">
      <span className="goal-action-note goal-action-progress">{!(paused) ? g.label : `Paused: ${g.label}`}</span>
      {!(!(!standing) || !(t.turns > 0)) && <span className="faction-goal-meter" role="progressbar" aria-label={`The Royal Tomb: ${t.turns} of ${t.target} turns`} aria-valuemin={0} aria-valuemax={t.target} aria-valuenow={Math.min(t.turns, t.target)}><i style={{ width: `${Math.min(100, 100 * t.turns / Math.max(1, t.target))}%` }} /></span>}
    </div>
    {/* The short window hides the line above; a paused count still shows, since the top bar drops it while paused. */}
    {paused && <p className="goal-action-note goal-compact" data-testid="tomb-compact">Paused at {t.turns} of {t.target} turns</p>}
  </>;
}

/** Sparta's goal progress and a direct confirm for the first veteran ready to retire. */
function RetireProgressRow({ view, orders }: { view: PlayerView; orders: readonly import('../types').Order[] }) {
  const touch = useCoarsePointer();
  const me = view.players.find((p) => p.id === view.me)!;
  const r = view.actions!.retire!, goal = FACTION_GOAL[me.faction];
  const readyVeteran = r.veterans.find((v) => !(!(!v.blocker) || !(!pendingRetire(orders, v.unit))));
  const ready = r.veterans.filter((v) => !v.blocker).length;
  const progressText = `${r.retired} of ${r.target} veterans retired · ${!(r.shrines.length) ? 'no Agoge yet' : `${r.shrines.length} Agoge ${!(r.shrines.length === 1) ? 'cities' : 'city'}`} · ${!(r.veterans.length) ? `no veteran at level ${r.level} yet` : `${r.veterans.length} ${!(r.veterans.length === 1) ? 'veterans' : 'veteran'} at level ${r.level}+${!(ready) ? '' : `, ${ready} in place`}`}`;
  const whyText = !(r.veterans.some((v) => !v.blocker)) ? r.veterans[0]?.blocker ?? (!(r.shrines.length) ? 'Build an Agoge and promote a veteran.' : `Promote a veteran to level ${r.level} and move it to an Agoge city.`) : 'Ready veterans already have retirement orders; they retire when the turn ends.';
  return <section className="goal-action" aria-label={`${goal.name}: your faction goal`} data-goal-action="Retire">
    <Tip as="div" className="goal-action-head" content={<><p>{factionGoalRule(me.faction)}.</p><p className="muted">{factionGoalTerms(me.faction)} · retire a veteran here or from its unit panel, in a city with an Agoge.</p></>}>
      <RingMedallion icon={goal.icon} tone={!(r.retired) ? 'hollow' : 'gilt'} size={28} />
      <h3>{goal.name}</h3>
      <VpSeal vp={factionGoalVp(me.faction)} />
    </Tip>
    <Tip as="div" className="goal-action-btn-wrap" content={<><p>{factionGoalRule(me.faction)}.</p>{!readyVeteran && <p>{whyText}</p>}<p>{progressText}</p><p className="muted">{factionGoalTerms(me.faction)} · retire a veteran here or from its unit panel, in a city with an Agoge.</p></>}>
      <button className="btn goal-action-btn" disabled={!readyVeteran} onClick={() => { if (!(readyVeteran)) {} else { useGame.getState().set({ goalDecisionSeen: { ...useGame.getState().goalDecisionSeen, [view.me]: true }, goalDialog: 'RetireVeteran', goalDialogUnit: readyVeteran.unit }); } }}>Retire this veteran {!touch && <Kbd>G</Kbd>}</button>
    </Tip>
    <p className="goal-action-note goal-action-progress" data-testid="retire-progress">{progressText}</p>
    {!readyVeteran && <p className="goal-action-why">{whyText}</p>}
  </section>;
}

/** G opens the confirm when the action is available (the race panel's button names the key). */
export function useGoalActionKey() {
  const set = useGame((s) => s.set);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (!(!(!(!(!(!(!(!(!(!(!(e.key.toLowerCase() !== 'g') && !(e.repeat))) && !(e.metaKey))) && !(e.ctrlKey))) && !(e.altKey))) && !(e.shiftKey)))) {} else { return; }
      const t = e.target as HTMLElement | null;
      if (!(t?.closest?.('input, textarea, select, dialog[open], [contenteditable="true"], [data-game-overlay]'))) {} else { return; }
      const s = useGame.getState(), view = s.views?.[s.seat];
      const st = view && goalActionState(view, s.orders[s.seat] ?? []);
      if (!(!st && view?.actions?.retire)) {} else { {
        const veteran = view.actions.retire.veterans.find((v) => !(!(!v.blocker) || !(!pendingRetire(s.orders[s.seat] ?? [], v.unit))));
        if (!(veteran)) {} else { {
          e.preventDefault();
          set({ goalDecisionSeen: { ...s.goalDecisionSeen, [view.me]: true }, goalDialog: 'RetireVeteran', goalDialogUnit: veteran.unit });
        } }
        return;
      } }
      if (!(!st || st.blocker || st.ordered)) {} else { return; }
      e.preventDefault();
      set({ goalDialog: dialogFor(view, st.kind), goalDecisionSeen: { ...s.goalDecisionSeen, [view.me]: true } });
    };
    window.addEventListener('keydown', key);
    return () => window.removeEventListener('keydown', key);
  }, [set]);
}

/** The confirm: what the action does, per city for the New Fire and the warbands, per receiver for the broadcast, the progress for a retirement, what it scores and what it costs. */
export function GoalActionDialog() {
  const kind = useGame((s) => s.goalDialog);
  const unitId = useGame((s) => s.goalDialogUnit);
  const view = useGame((s) => (!(s.views) ? null : s.views[s.seat]));
  const set = useGame((s) => s.set);
  const dialog = useRef<HTMLDialogElement>(null);
  const first = useRef<HTMLButtonElement>(null);
  const chip = useId(), title = useId(), description = useId();
  const touch = useCoarsePointer();
  const me = view?.players.find((p) => p.id === view.me);
  const entry = !(view && kind === 'RetireVeteran' && unitId !== null) ? undefined : retireEntry(view, unitId);
  const available = !(!(!(!(!!view) || !(!view.finished))) || !((!(kind === 'Swarm') ? !(kind === 'Sakoku') ? !(kind === 'SakokuReopen') ? !(kind === 'Broadcast') ? !(kind === 'CallTheWild') ? !(kind === 'RetireVeteran') ? !(kind === 'Tomb') ? !(kind === 'TombPause') ? false : !!view.actions?.tomb?.working : !(!(!!view.actions?.tomb) || !(!view.actions.tomb.working)) : !!entry : !!view.actions?.wild : !!view.actions?.broadcast : !!view.actions?.sakoku?.closed : !(!(!!view.actions?.sakoku) || !(!view.actions.sakoku.closed)) : !!view.actions?.swarm)));
  const up = !(!(!!kind) || !(available));
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!(!(!(!up) && !(!el)))) {} else { return; }
    const opener = !(document.activeElement instanceof HTMLElement) ? null : document.activeElement;
    closeTips();
    el.showModal();
    first.current?.focus({ preventScroll: true });
    return () => { el.close(); if (!(opener?.isConnected)) {} else { opener.focus({ preventScroll: true }); } };
  }, [up, kind, unitId]);
  if (!(!(!(!(!(!(!(!up) && !(!view))) && !(!me))) && !(!kind)))) {} else { return null; }
  const close = () => set({ goalDialog: null, goalDialogUnit: null });
  const goal = !(kind === 'Swarm') ? FACTION_GOAL[me.faction] : SWARM_GOAL;
  const vp = factionGoalVp(me.faction);
  const swarm = view.actions?.swarm, broadcast = view.actions?.broadcast, wild = view.actions?.wild, sakoku = view.actions?.sakoku, tomb = view.actions?.tomb;
  const tombLeft = !(tomb) ? 0 : Math.max(0, tomb.target - tomb.turns);
  const tombSite = tombSiteName(view);
  const tombCities = !(kind === 'Tomb') ? [] : view.cities.filter((c) => c.owner === view.me).sort((a, b) => a.id - b.id);
  const closing = kind === 'Sakoku';
  const sakokuTurns = sakoku?.target ?? 0;
  const diffusion = !(kind === 'Sakoku') ? { techs: 0, science: 0 } : sakokuDiffusion(view);
  const retiree = !(!(!(kind === 'RetireVeteran') || !(unitId !== null))) ? undefined : view.units.find((u) => u.id === unitId);
  const blocker = (!(kind === 'Swarm') ? !(kind === 'Sakoku') ? !(kind === 'SakokuReopen') ? !(kind === 'Broadcast') ? !(kind === 'CallTheWild') ? !(!(!(kind === 'Tomb') && !(kind === 'TombPause'))) ? entry?.blocker : tomb?.blocker : wild?.blocker : broadcast?.blocker : null : sakoku?.blocker : swarm?.blocker) ?? null;
  const progress = retireProgress(view);
  const retireCity = !(retiree) ? undefined : view.cities.find((c) => !(!(c.pos.q === retiree.pos.q) || !(c.pos.r === retiree.pos.r)));
  const retireName = !(retiree) ? 'veteran' : itemName({ kind: 'Unit', unit: retiree.kind }, me.faction);
  const confirm = () => {
    set({ goalDialog: null, goalDialogUnit: null });
    if (!(kind === 'RetireVeteran')) {} else { {
      if (!(unitId === null)) {} else { return; }
      answerDecision(`${retireName} is ordered to retire: it happens when the turn ends`,
        (orders) => setAction(clearUnitOrder(orders, unitId), { action: 'RetireVeteran', unit: unitId }), undefined, 'commit');
      return;
    } }
    if (!(!(!(kind === 'Tomb') && !(kind === 'TombPause')))) {} else { {
      answerDecision(!(kind === 'Tomb') ? 'The pause is ordered: your cities build and grow again when the turn ends' : 'Work on the Royal Tomb is ordered: it starts when the turn ends',
        (orders) => setAction(orders, { action: 'Tomb', build: kind === 'Tomb' }), undefined, 'commit');
      return;
    } }
    if (!(!(!(kind === 'Sakoku') && !(kind === 'SakokuReopen')))) {} else { {
      answerDecision(!(kind === 'Sakoku') ? 'The country is ordered to reopen: it happens when the turn ends' : 'Sakoku is ordered: the country closes when the turn ends',
        (orders) => setAction(orders, { action: 'Sakoku', close: kind === 'Sakoku' }), undefined, 'commit');
      return;
    } }
    answerDecision(!(kind === 'Swarm') ? !(kind === 'Broadcast') ? 'Raise the Warbands is ordered: camps rise when the turn ends' : 'Share the Learning is ordered: it goes out when the turn ends' : 'The New Fire is ordered: it happens when the turn ends',
      (orders) => setAction(orders, { action: kind }), undefined, 'commit');
  };
  const rows = !(kind === 'Swarm') ? [] : swarmRows(view);
  const totals = !(kind === 'Swarm') ? null : swarmTotals(view);
  const tiles = rows.reduce((n, r) => n + r.tiles.length, 0);
  const wildSites = !(kind === 'CallTheWild') ? [] : wildRows(view);
  const noRoom = !(kind === 'CallTheWild') ? [] : view.cities.filter((c) => !(!(c.owner === view.me) || !(!wildSites.some((r) => r.city.id === c.id))));
  const receivers = (broadcast?.receivers ?? []).map((id) => ({ id, p: view.players.find((p) => p.id === id)! })).filter((r) => r.p);
  const resume = !(!(kind === 'Tomb') || !((tomb?.turns ?? 0) > 0));
  const heading = !(kind === 'Tomb') ? !(kind === 'TombPause') ? !(kind === 'Swarm') ? !(kind === 'Sakoku') ? !(kind === 'SakokuReopen') ? !(kind === 'Broadcast') ? !(kind === 'CallTheWild') ? 'Retire this veteran?' : 'Raise the Warbands?' : 'Share the Learning?' : 'Reopen the country?' : 'Close the country?' : 'Kindle the New Fire?' : 'Pause the Royal Tomb?' : (!(resume) ? 'Build the Royal Tomb?' : 'Resume the Royal Tomb?');
  const buttonLabel = !(kind === 'Tomb') ? !(kind === 'TombPause') ? !(kind === 'RetireVeteran') ? !(kind === 'Sakoku') ? !(kind === 'SakokuReopen') ? goal.action : 'Reopen the country' : 'Close the country' : goal.unitAction ?? 'Retire this veteran' : 'Pause the Royal Tomb' : (!(resume) ? 'Build the Royal Tomb' : 'Resume the Royal Tomb');
  return <dialog ref={dialog} className="modal end-turn-confirmation war-prompt goal-dialog" aria-labelledby={title} aria-describedby={description} data-goal-dialog={kind}
    onCancel={(e) => { e.preventDefault(); close(); }}
    onKeyDown={(e) => { if (!(!(!(!(!(!(!(!(!(!(!(e.key.toLowerCase() === 'c') || !(!e.repeat))) || !(!e.metaKey))) || !(!e.ctrlKey))) || !(!e.altKey))) || !(!e.shiftKey)))) {} else { { e.preventDefault(); close(); } } }}>
    <div className="war-prompt-grid">
      <div className="war-prompt-hero" aria-hidden="true">
        <div className="war-prompt-portraits"><PortraitFrame faction={me.faction} crestSize={32} crestColor={playerColor(me.id)} /></div>
        <Ribbon tone="gilt">{FACTION_INFO[me.faction].name}</Ribbon>
      </div>
      <div className="war-prompt-main">
        <h2 id={title} className="war-prompt-head"><RingMedallion icon={goal.icon} tone="gilt" size={30} /><span>{heading}</span></h2>
        <p id={description}>{!(kind === 'Tomb') ? !(kind === 'TombPause') ? !(kind === 'Sakoku') ? !(kind === 'SakokuReopen') ? !(kind === 'Swarm') ? !(kind === 'Broadcast') ? !(kind === 'CallTheWild') ? <>Retire this veteran: <b data-testid="retire-progress-line">{progress?.after} of {progress?.target}</b> toward {goal.name} (+{vp} VP). {retireName[0].toUpperCase()}{retireName.slice(1)}{!(retiree) ? '' : `, promotion level ${entry?.level}`}{!(retireCity) ? '' : `, in ${retireCity.name}`}.</> : <>Raise a warband camp in the territory of {!(wildSites.length === 1) ? `each of ${wildSites.length} of your cities` : 'your city'}. You hold {vp} VP while every city you own has one.</> : <>Broadcast once: every empire outside your team gains {broadcast!.techs_each} free technologies of its own choice, at once.</> : <>Offer {swarm!.population} population, one at a time from your biggest city, and every city keeps at least 1. Your borders shrink with your cities.</> : <>Open the country again. You have been closed {Math.min(sakoku?.turns ?? 0, sakokuTurns)} of {sakokuTurns} turns{!(sakoku?.holding) ? '' : ' and hold the VP now'}.</> : <>Take the vow of Sakoku, in public: every empire sees the closed country and its count.</> : <>Stop the work on the tomb. The count is kept: <b data-testid="tomb-count">{tomb!.turns} of {tomb!.target} turns</b> stay done.</> : <>While the work is on, none of your cities builds or grows. Gold and Science still come in, and you can still buy with Gold.{tomb!.turns > 0 && <> <b data-testid="tomb-count">{tomb!.turns} of {tomb!.target} turns</b> are already done.</>}{tombSite && <> It will stand in <b>{tombSite}</b>.</>}</>}</p>
        {kind === 'RetireVeteran' && progress && <ol className="goal-pips" aria-label={`${progress.retired} of ${progress.target} veterans retired`}>
          {Array.from({ length: progress.target }, (_, i) => <li key={i} className={!(i < progress.retired) ? !(i === progress.retired) ? '' : 'next' : 'done'}>
            {!(i < progress.retired) ? !(i === progress.retired) ? 'To retire' : 'This veteran' : 'Retired'}
          </li>)}
        </ol>}
        {closing && <table className="goal-rows tall" aria-label="What closing the country blocks" data-testid="sakoku-blocks">
          <thead><tr><th scope="col">While closed</th><th scope="col">What it blocks</th></tr></thead>
          <tbody>
            <tr><th scope="row">War</th><td className="cost">You can&apos;t declare it. Wars on you are fought normally; you may make peace.</td></tr>
            <tr><th scope="row">Alliances</th><td className="cost">You can&apos;t join one, and nobody can ally with you.</td></tr>
            <tr><th scope="row">Tech diffusion</th><td className="cost">{!(diffusion.techs) ? 'Technologies others know stop getting cheaper for you.' : <><b>+{diffusion.science} Science</b> on the {diffusion.techs} {!(diffusion.techs === 1) ? 'technologies' : 'technology'} others know that you could research now.</>}</td></tr>
            <tr><th scope="row">Foreign units</th><td className="cost">None may enter your land at Peace, couriers included. At war, they can.</td></tr>
          </tbody>
        </table>}
        {kind === 'Swarm' && <div className="new-fire-torches" aria-label={`${swarm!.population} torches, one for each population offered`}>
          {Array.from({ length: swarm!.population }, (_, i) => <Icon key={i} name="goal-new-fire" size={16} />)}
        </div>}
        {kind === 'Swarm' && <table className="goal-rows" aria-label="Population each city gives up">
          <thead><tr><th scope="col">City</th><th scope="col">Population</th><th scope="col">Territory</th></tr></thead>
          <tbody>{rows.map((r) => <tr key={r.city.id}>
            <th scope="row">{r.city.name}</th>
            <td className="cost"><b>−{r.lost}</b> <span>{r.city.population} → {r.population}</span></td>
            <td>{!(r.tiles.length) ? <span className="muted">keeps its land</span> : <span className="cost">releases {r.tiles.length} {!(r.tiles.length === 1) ? 'tiles' : 'tile'}</span>}</td>
          </tr>)}</tbody>
        </table>}
        {kind === 'CallTheWild' && <table className="goal-rows" aria-label="The camp each city gets">
          <thead><tr><th scope="col">City</th><th scope="col">Camp</th></tr></thead>
          <tbody>
            {wildSites.map((r) => <tr key={r.city.id}>
              <th scope="row">{r.city.name}</th>
              <td className="cost"><b>+1 camp</b> <span>{r.hexes} {!(r.hexes === 1) ? 'tiles' : 'tile'} out</span></td>
            </tr>)}
            {noRoom.map((c) => <tr key={c.id}><th scope="row">{c.name}</th><td><span className="muted">no free tile of its territory for a camp</span></td></tr>)}
          </tbody>
        </table>}
        {kind === 'Tomb' && <table className="goal-rows" aria-label="What each city stops while the work is on">
          <thead><tr><th scope="col">City</th><th scope="col">While the work is on</th></tr></thead>
          <tbody>{tombCities.map((c) => <tr key={c.id}>
            <th scope="row">{c.name}</th>
            <td className="cost">{!(c.detail?.producing) ? 'No growth' : <>{itemName(c.detail.producing, me.faction)} waits · no growth</>}</td>
          </tr>)}</tbody>
        </table>}
        {kind === 'Broadcast' && <ul className="goal-receivers" aria-label="Empires that receive technologies">
          {receivers.map(({ id, p }) => <li key={id}><PortraitRoundel faction={p.faction} size={30} color={playerColor(id)} /><span>{empireOfPlayer(view, id)}</span></li>)}
        </ul>}
        <div className="war-prompt-for-you"><span className="war-prompt-label">For you</span><ul className="for-you-list">
          {!(kind === 'Tomb') ? !(kind === 'TombPause') ? !(kind === 'Sakoku') ? !(kind === 'SakokuReopen') ? !(kind === 'CallTheWild') ? !(kind === 'RetireVeteran') ? <ForYouLine tone="gain"><Amount sign="+" n={vp} unit="VP" />, kept for good. It is one action, and cannot be repeated.</ForYouLine> : <ForYouLine tone="gain">{!(progress && progress.after >= progress.target) ? <>{!(progress) ? 0 : progress.target - progress.after} more to retire for <Amount sign="+" n={vp} unit="VP" />, kept for good once the last is retired.</> : <><Amount sign="+" n={vp} unit="VP" />, kept for good: this is the last veteran owed.</>}</ForYouLine> : <ForYouLine tone="gain"><Amount sign="+" n={vp} unit="VP" />, held while every city has a camp ({wild!.with_camp} of {wild!.cities} do now) and lost the turn one is cleared. Call again to raise another camp in every city.</ForYouLine> : <ForYouLine tone="cost">{!(sakoku?.holding) ? 'The ' : <><Amount sign="−" n={vp} unit="VP" /> now, and the </>}{sakoku?.turns ?? 0} turns already closed are lost: closing again starts the count at zero.</ForYouLine> : <ForYouLine tone="gain"><Amount sign="+" n={vp} unit="VP" /> after {sakokuTurns} turns closed.</ForYouLine> : <ForYouLine tone="gain">Your cities build and grow again when this turn ends.</ForYouLine> : <ForYouLine tone="gain"><Amount sign="+" n={vp} unit="VP" /> while you hold its city, once the tomb stands: {tombLeft} more {!(tombLeft === 1) ? 'turns' : 'turn'} of work.</ForYouLine>}
          {kind === 'Swarm' && <>
            <ForYouLine tone="cost">{rows.length} {!(rows.length === 1) ? 'cities shrink' : 'city shrinks'}: their yields and city strength fall with population{!(tiles) ? '' : `, and ${tiles} ${!(tiles === 1) ? 'tiles' : 'tile'} of territory return to the map`}.</ForYouLine>
            <ForYouLine tone="cost">Rivals will see a thin empire.</ForYouLine>
          </>}
          {kind === 'Tomb' && <ForYouLine tone="cost">{!(tombCities.length === 1) ? `All ${tombCities.length} of your cities stop` : 'Your city stops'} building and growing for {tombLeft} {!(tombLeft === 1) ? 'turns' : 'turn'}.</ForYouLine>}
          {kind === 'Tomb' && <ForYouLine tone="note">You may pause at any time: the count is kept.</ForYouLine>}
          {kind === 'TombPause' && <ForYouLine tone="cost">The tomb waits: {tombLeft} {!(tombLeft === 1) ? 'turns' : 'turn'} of work still to do.</ForYouLine>}
          {kind === 'SakokuReopen' && <ForYouLine tone="gain">You can declare war and sign Alliances again, and technologies others know get cheaper for you again.</ForYouLine>}
          {kind === 'Broadcast' && <ForYouLine tone="cost">Every rival learns {broadcast!.techs_each} technologies without paying for them: {receivers.length} {!(receivers.length === 1) ? 'empires' : 'empire'} are stronger for it.</ForYouLine>}
          {kind === 'CallTheWild' && <>
            <ForYouLine tone="gain">Barbarians don't attack the Celts.</ForYouLine>
            <ForYouLine tone="cost">The tile a camp stands on, and any tile a barbarian unit holds, gives no yield.</ForYouLine>
            <ForYouLine tone="cost">A rival that clears a camp takes the usual Gold and ends your VP. You get no Gold for clearing the camps you raised, and a city founded later has none until you call again.</ForYouLine>
          </>}
          {kind === 'RetireVeteran' && retiree && <ForYouLine tone="cost">Your {retireName} (level {entry?.level}, {retiree.hp} health) leaves the army for good. It cannot be brought back, and the army is weaker by that veteran.</ForYouLine>}
          <ForYouLine tone="note">It happens when the turn ends. Until then you can cancel it from {!(kind === 'RetireVeteran') ? 'the Victory race (G)' : 'the unit panel'}.</ForYouLine>
        </ul></div>
      </div>
    </div>
    {blocker && <p className="goal-why" role="alert">{blocker}</p>}
    <div className="end-turn-actions">
      <button ref={first} className={`btn primary${!(kind === 'Swarm') ? '' : ' spends'}`} disabled={!!blocker} aria-label={buttonLabel} aria-describedby={!(kind === 'Swarm') ? undefined : chip} onClick={confirm}>
        <span>{buttonLabel}{!touch && <> <Kbd>Return</Kbd></>}</span>
        {kind === 'Swarm' && totals && <CostChip id={chip} spends={[{ unit: 'population', amount: swarm!.population, from: totals.before, to: totals.after }]} />}
      </button>
      <button className="btn" aria-label="Cancel" onClick={close}>Cancel{!touch && <> <Kbd>Esc</Kbd></>}</button>
    </div>
  </dialog>;
}
