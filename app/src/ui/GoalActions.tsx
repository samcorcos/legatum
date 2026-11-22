// The goal rites that are one-time actions: the Swarm (the Aztecs' New Fire), Japan's Sakoku vow, the Athens' broadcast, the
// Celts's Call the warbands and Sparta's retirement of a veteran. The race panel carries the
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

type Kind = 'Swarm' | 'Sakoku' | 'Broadcast' | 'CallTheWild';

/** Which of the viewer's goal actions exists (the faction's) and whether it can be ordered. */
function goalActionState(view: PlayerView, orders: readonly import('../types').Order[]) {
  const swarm = view.actions?.swarm, sakoku = view.actions?.sakoku, broadcast = view.actions?.broadcast, wild = view.actions?.wild;
  const kind: Kind | null = swarm ? 'Swarm' : sakoku ? 'Sakoku' : broadcast ? 'Broadcast' : wild ? 'CallTheWild' : null;
  if (!kind) return null;
  const blocker = (kind === 'Swarm' ? swarm?.blocker : kind === 'Sakoku' ? sakoku?.blocker : kind === 'Broadcast' ? broadcast?.blocker : wild?.blocker) ?? null;
  return { kind, blocker, ordered: !!pendingAction(orders, kind) };
}

/** The dialog an action opens: Sakoku is one action that closes an open country and reopens a closed one. */
const dialogFor = (view: PlayerView, kind: Kind): 'Swarm' | 'Sakoku' | 'SakokuReopen' | 'Broadcast' | 'CallTheWild' =>
  kind === 'Sakoku' ? (view.actions?.sakoku?.closed ? 'SakokuReopen' : 'Sakoku') : kind;

/** The race panel's own-goal action: what it is, what it scores, and the one button to take it. */
export function GoalActionRow() {
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const set = useGame((s) => s.set);
  const editOrders = useGame((s) => s.editOrders);
  const touch = useCoarsePointer();
  const me = view?.players.find((p) => p.id === view.me);
  if (!view || !me || view.finished || !me.alive) return null;
  const st = goalActionState(view, orders);
  if (!st) return view.actions?.retire ? <RetireProgressRow view={view} orders={orders} /> : null;
  const goal = st.kind === 'Swarm' ? SWARM_GOAL : FACTION_GOAL[me.faction];
  const spare = view.actions?.swarm, wild = view.actions?.wild, sakoku = view.actions?.sakoku;
  const why = st.blocker && !st.ordered ? st.blocker : null;
  const label = st.kind === 'Sakoku' ? (sakoku?.closed ? 'Reopen the country' : 'Close the country') : goal.action;
  const goalName = st.kind === 'Sakoku' ? 'Sakoku' : goal.name;
  const rule = st.kind === 'Sakoku' ? `Close the country for ${sakoku?.target ?? 30} turn ends to hold +${factionGoalVp(me.faction)} VP` : factionGoalRule(me.faction);
  const terms = st.kind === 'Sakoku' ? 'Reopening loses the held VP' : factionGoalTerms(me.faction);
  const progress = st.kind === 'Sakoku' ? view.faction_goal?.label : wild ? `${wild.with_camp} of ${wild.cities} ${wild.cities === 1 ? 'city has' : 'cities have'} a wild camp in ${wild.cities === 1 ? 'its' : 'their'} territory.` : null;
  const whyText = why ? (st.kind === 'Swarm' && spare ? `${spare.spare} of ${RULES.swarmPopulation} population to spare: every city keeps 1.` : why) : null;
  // The button's hover carries everything the short window's one-line row leaves out: the rule, the reason it is blocked and the progress.
  const buttonTip = <><p>{rule}.</p>{whyText && <p>{whyText}</p>}{progress && <p>{progress}</p>}<p className="muted">{terms} · one action, confirmed first, resolved when the turn ends.</p></>;
  return <section className={`goal-action${st.ordered ? ' ordered' : ''}${why ? ' blocked' : ''}`} aria-label={`${goalName}: your faction goal`} data-goal-action={st.kind}>
    <Tip as="div" className="goal-action-head" content={<><p>{rule}.</p><p className="muted">{terms} · one action, confirmed first, resolved when the turn ends.</p></>}>
      <RingMedallion icon={goal.icon} tone={st.ordered ? 'gilt' : 'hollow'} size={28} />
      <h3>{goalName}</h3>
      <VpSeal vp={factionGoalVp(me.faction)} />
    </Tip>
    <Tip as="div" className="goal-action-btn-wrap" content={buttonTip}>
      {st.ordered
        ? <button className="btn goal-action-btn" onClick={() => editOrders((o) => clearAction(o, st.kind))}>Cancel the order</button>
        : <button className="btn goal-action-btn" disabled={!!st.blocker} onClick={() => { closeTips(); set({ goalDialog: dialogFor(view, st.kind), goalDecisionSeen: { ...useGame.getState().goalDecisionSeen, [view.me]: true } }); }}>
          {label}{!touch && <> <Kbd>G</Kbd></>}
        </button>}
    </Tip>
    {sakoku && <SakokuMeter view={view} />}
    {wild && <p className="goal-action-note goal-action-progress" data-testid="wild-progress">{progress}</p>}
    {st.ordered && <p className="goal-action-note" role="status">Ordered: it resolves when the turn ends.</p>}
    {whyText && <p className="goal-action-why">{whyText}</p>}
  </section>;
}

/** Sakoku's countdown in the race panel's row: turns closed against the turns needed, and what keeps the VP away. */
function SakokuMeter({ view }: { view: PlayerView }) {
  const g = view.faction_goal, s = view.actions!.sakoku!;
  if (!g) return null;
  return <div className="goal-action-sakoku" data-testid="sakoku-progress">
    <span className="goal-action-note goal-action-progress">{g.label}</span>
    {s.closed && <span className="faction-goal-meter" role="progressbar" aria-label={`Sakoku: ${g.label}`} aria-valuemin={0} aria-valuemax={s.target} aria-valuenow={Math.min(s.turns, s.target)}><i style={{ width: `${Math.min(100, 100 * s.turns / Math.max(1, s.target))}%` }} /></span>}
  </div>;
}

/** Sparta's goal progress and a direct confirm for the first veteran ready to retire. */
function RetireProgressRow({ view, orders }: { view: PlayerView; orders: readonly import('../types').Order[] }) {
  const touch = useCoarsePointer();
  const me = view.players.find((p) => p.id === view.me)!;
  const r = view.actions!.retire!, goal = FACTION_GOAL[me.faction];
  const readyVeteran = r.veterans.find((v) => !v.blocker && !pendingRetire(orders, v.unit));
  const ready = r.veterans.filter((v) => !v.blocker).length;
  const progressText = `${r.retired} of ${r.target} veterans retired · ${r.shrines.length ? `${r.shrines.length} Agoge ${r.shrines.length === 1 ? 'city' : 'cities'}` : 'no Agoge yet'} · ${r.veterans.length ? `${r.veterans.length} ${r.veterans.length === 1 ? 'veteran' : 'veterans'} at level ${r.level}+${ready ? `, ${ready} in place` : ''}` : `no veteran at level ${r.level} yet`}`;
  const whyText = r.veterans.some((v) => !v.blocker) ? 'Ready veterans already have retirement orders; they resolve when the turn ends.' : r.veterans[0]?.blocker ?? (r.shrines.length ? `Promote a veteran to level ${r.level} and move it to an Agoge city.` : 'Build an Agoge and promote a veteran.');
  return <section className="goal-action" aria-label={`${goal.name}: your faction goal`} data-goal-action="Retire">
    <Tip as="div" className="goal-action-head" content={<><p>{factionGoalRule(me.faction)}.</p><p className="muted">{factionGoalTerms(me.faction)} · retire a veteran here or from its unit panel, in a city with an Agoge.</p></>}>
      <RingMedallion icon={goal.icon} tone={r.retired ? 'gilt' : 'hollow'} size={28} />
      <h3>{goal.name}</h3>
      <VpSeal vp={factionGoalVp(me.faction)} />
    </Tip>
    <Tip as="div" className="goal-action-btn-wrap" content={<><p>{factionGoalRule(me.faction)}.</p>{!readyVeteran && <p>{whyText}</p>}<p>{progressText}</p><p className="muted">{factionGoalTerms(me.faction)} · retire a veteran here or from its unit panel, in a city with an Agoge.</p></>}>
      <button className="btn goal-action-btn" disabled={!readyVeteran} onClick={() => { if (readyVeteran) useGame.getState().set({ goalDecisionSeen: { ...useGame.getState().goalDecisionSeen, [view.me]: true }, goalDialog: 'RetireVeteran', goalDialogUnit: readyVeteran.unit }); }}>Retire this veteran {!touch && <Kbd>G</Kbd>}</button>
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
      if (e.key.toLowerCase() !== 'g' || e.repeat || e.metaKey || e.ctrlKey || e.altKey || e.shiftKey) return;
      const t = e.target as HTMLElement | null;
      if (t?.closest?.('input, textarea, select, dialog[open], [contenteditable="true"], [data-game-overlay]')) return;
      const s = useGame.getState(), view = s.views?.[s.seat];
      const st = view && goalActionState(view, s.orders[s.seat] ?? []);
      if (!st && view?.actions?.retire) {
        const veteran = view.actions.retire.veterans.find((v) => !v.blocker && !pendingRetire(s.orders[s.seat] ?? [], v.unit));
        if (veteran) {
          e.preventDefault();
          set({ goalDecisionSeen: { ...s.goalDecisionSeen, [view.me]: true }, goalDialog: 'RetireVeteran', goalDialogUnit: veteran.unit });
        }
        return;
      }
      if (!st || st.blocker || st.ordered) return;
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
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const set = useGame((s) => s.set);
  const dialog = useRef<HTMLDialogElement>(null);
  const first = useRef<HTMLButtonElement>(null);
  const chip = useId(), title = useId(), description = useId();
  const touch = useCoarsePointer();
  const me = view?.players.find((p) => p.id === view.me);
  const entry = view && kind === 'RetireVeteran' && unitId !== null ? retireEntry(view, unitId) : undefined;
  const available = !!view && !view.finished && (kind === 'Swarm' ? !!view.actions?.swarm : kind === 'Sakoku' ? !!view.actions?.sakoku && !view.actions.sakoku.closed : kind === 'SakokuReopen' ? !!view.actions?.sakoku?.closed : kind === 'Broadcast' ? !!view.actions?.broadcast
    : kind === 'CallTheWild' ? !!view.actions?.wild : kind === 'RetireVeteran' ? !!entry : false);
  const up = !!kind && available;
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!up || !el) return;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    closeTips();
    el.showModal();
    first.current?.focus({ preventScroll: true });
    return () => { el.close(); if (opener?.isConnected) opener.focus({ preventScroll: true }); };
  }, [up, kind, unitId]);
  if (!up || !view || !me || !kind) return null;
  const close = () => set({ goalDialog: null, goalDialogUnit: null });
  const goal = kind === 'Swarm' ? SWARM_GOAL : FACTION_GOAL[me.faction];
  const vp = factionGoalVp(me.faction);
  const swarm = view.actions?.swarm, broadcast = view.actions?.broadcast, wild = view.actions?.wild, sakoku = view.actions?.sakoku;
  const closing = kind === 'Sakoku';
  const sakokuTurns = sakoku?.target ?? 0;
  const diffusion = kind === 'Sakoku' ? sakokuDiffusion(view) : { techs: 0, science: 0 };
  const retiree = kind === 'RetireVeteran' && unitId !== null ? view.units.find((u) => u.id === unitId) : undefined;
  const blocker = (kind === 'Swarm' ? swarm?.blocker : kind === 'Sakoku' ? sakoku?.blocker : kind === 'SakokuReopen' ? null : kind === 'Broadcast' ? broadcast?.blocker : kind === 'CallTheWild' ? wild?.blocker : entry?.blocker) ?? null;
  const progress = retireProgress(view);
  const retireCity = retiree ? view.cities.find((c) => c.pos.q === retiree.pos.q && c.pos.r === retiree.pos.r) : undefined;
  const retireName = retiree ? itemName({ kind: 'Unit', unit: retiree.kind }, me.faction) : 'veteran';
  const confirm = () => {
    set({ goalDialog: null, goalDialogUnit: null });
    if (kind === 'RetireVeteran') {
      if (unitId === null) return;
      answerDecision(`${retireName} is ordered to retire: it resolves when the turn ends`,
        (orders) => setAction(clearUnitOrder(orders, unitId), { action: 'RetireVeteran', unit: unitId }), undefined, 'commit');
      return;
    }
    if (kind === 'Sakoku' || kind === 'SakokuReopen') {
      answerDecision(kind === 'Sakoku' ? 'Sakoku is ordered: the country closes when the turn ends' : 'The country is ordered to reopen: it resolves when the turn ends',
        (orders) => setAction(orders, { action: 'Sakoku', close: kind === 'Sakoku' }), undefined, 'commit');
      return;
    }
    answerDecision(kind === 'Swarm' ? 'The New Fire is ordered: it resolves when the turn ends'
      : kind === 'Broadcast' ? 'Share the Learning is ordered: it goes out when the turn ends' : 'Raise the Warbands is ordered: camps rise when the turn ends',
      (orders) => setAction(orders, { action: kind }), undefined, 'commit');
  };
  const rows = kind === 'Swarm' ? swarmRows(view) : [];
  const totals = kind === 'Swarm' ? swarmTotals(view) : null;
  const tiles = rows.reduce((n, r) => n + r.tiles.length, 0);
  const wildSites = kind === 'CallTheWild' ? wildRows(view) : [];
  const noRoom = kind === 'CallTheWild' ? view.cities.filter((c) => c.owner === view.me && !wildSites.some((r) => r.city.id === c.id)) : [];
  const receivers = (broadcast?.receivers ?? []).map((id) => ({ id, p: view.players.find((p) => p.id === id)! })).filter((r) => r.p);
  const heading = kind === 'Swarm' ? 'Kindle the New Fire?' : kind === 'Sakoku' ? 'Close the country?' : kind === 'SakokuReopen' ? 'Reopen the country?' : kind === 'Broadcast' ? 'Share the Learning?' : kind === 'CallTheWild' ? 'Raise the Warbands?' : 'Retire this veteran?';
  const buttonLabel = kind === 'RetireVeteran' ? goal.unitAction ?? 'Retire this veteran' : kind === 'Sakoku' ? 'Close the country' : kind === 'SakokuReopen' ? 'Reopen the country' : goal.action;
  return <dialog ref={dialog} className="modal end-turn-confirmation war-prompt goal-dialog" aria-labelledby={title} aria-describedby={description} data-goal-dialog={kind}
    onCancel={(e) => { e.preventDefault(); close(); }}
    onKeyDown={(e) => { if (e.key.toLowerCase() === 'c' && !e.repeat && !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey) { e.preventDefault(); close(); } }}>
    <div className="war-prompt-grid">
      <div className="war-prompt-hero" aria-hidden="true">
        <div className="war-prompt-portraits"><PortraitFrame faction={me.faction} crestSize={32} crestColor={playerColor(me.id)} /></div>
        <Ribbon tone="gilt">{FACTION_INFO[me.faction].name}</Ribbon>
      </div>
      <div className="war-prompt-main">
        <h2 id={title} className="war-prompt-head"><RingMedallion icon={goal.icon} tone="gilt" size={30} /><span>{heading}</span></h2>
        <p id={description}>{kind === 'Sakoku'
          ? <>Take the vow of Sakoku, in public: every empire sees the closed country and its count.</>
          : kind === 'SakokuReopen'
            ? <>Open the country again. You have been closed {Math.min(sakoku?.turns ?? 0, sakokuTurns)} of {sakokuTurns} turns{sakoku?.holding ? ' and hold the VP now' : ''}.</>
            : kind === 'Swarm'
          ? <>Offer {swarm!.population} population, one at a time from your biggest city, and every city keeps at least 1. Your borders shrink with your cities.</>
          : kind === 'Broadcast'
            ? <>Broadcast once: every empire outside your team gains {broadcast!.techs_each} free technologies of its own choice, at once.</>
            : kind === 'CallTheWild'
              ? <>Raise a warband camp in the territory of {wildSites.length === 1 ? 'your city' : `each of ${wildSites.length} of your cities`}. You hold {vp} VP while every city you own has one.</>
              : <>Retire this veteran: <b data-testid="retire-progress-line">{progress?.after} of {progress?.target}</b> toward {goal.name} (+{vp} VP). {retireName[0].toUpperCase()}{retireName.slice(1)}{retiree ? `, promotion level ${entry?.level}` : ''}{retireCity ? `, in ${retireCity.name}` : ''}.</>}</p>
        {kind === 'RetireVeteran' && progress && <ol className="goal-pips" aria-label={`${progress.retired} of ${progress.target} veterans retired`}>
          {Array.from({ length: progress.target }, (_, i) => <li key={i} className={i < progress.retired ? 'done' : i === progress.retired ? 'next' : ''}>
            {i < progress.retired ? 'Retired' : i === progress.retired ? 'This veteran' : 'To retire'}
          </li>)}
        </ol>}
        {closing && <table className="goal-rows tall" aria-label="What closing the country blocks" data-testid="sakoku-blocks">
          <thead><tr><th scope="col">While closed</th><th scope="col">What it blocks</th></tr></thead>
          <tbody>
            <tr><th scope="row">War</th><td className="cost">You can&apos;t declare it. Wars on you are fought normally; you may make peace.</td></tr>
            <tr><th scope="row">Alliances</th><td className="cost">You can&apos;t join one, and nobody can ally with you.</td></tr>
            <tr><th scope="row">Tech diffusion</th><td className="cost">{diffusion.techs ? <><b>+{diffusion.science} Science</b> on the {diffusion.techs} {diffusion.techs === 1 ? 'technology' : 'technologies'} others know that you could research now.</> : 'Technologies others know stop getting cheaper for you.'}</td></tr>
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
            <td>{r.tiles.length ? <span className="cost">releases {r.tiles.length} {r.tiles.length === 1 ? 'tile' : 'tiles'}</span> : <span className="muted">keeps its land</span>}</td>
          </tr>)}</tbody>
        </table>}
        {kind === 'CallTheWild' && <table className="goal-rows" aria-label="The camp each city gets">
          <thead><tr><th scope="col">City</th><th scope="col">Camp</th></tr></thead>
          <tbody>
            {wildSites.map((r) => <tr key={r.city.id}>
              <th scope="row">{r.city.name}</th>
              <td className="cost"><b>+1 camp</b> <span>{r.hexes} {r.hexes === 1 ? 'tile' : 'tiles'} out</span></td>
            </tr>)}
            {noRoom.map((c) => <tr key={c.id}><th scope="row">{c.name}</th><td><span className="muted">no free tile of its territory for a camp</span></td></tr>)}
          </tbody>
        </table>}
        {kind === 'Broadcast' && <ul className="goal-receivers" aria-label="Empires that receive technologies">
          {receivers.map(({ id, p }) => <li key={id}><PortraitRoundel faction={p.faction} size={30} color={playerColor(id)} /><span>{empireOfPlayer(view, id)}</span></li>)}
        </ul>}
        <div className="war-prompt-for-you"><span className="war-prompt-label">For you</span><ul className="for-you-list">
          {kind === 'Sakoku'
            ? <ForYouLine tone="gain"><Amount sign="+" n={vp} unit="VP" /> after {sakokuTurns} turns closed.</ForYouLine>
            : kind === 'SakokuReopen'
              ? <ForYouLine tone="cost">{sakoku?.holding ? <><Amount sign="−" n={vp} unit="VP" /> now, and the </> : 'The '}{sakoku?.turns ?? 0} turns already closed are lost: closing again starts the count at zero.</ForYouLine>
              : kind === 'CallTheWild'
            ? <ForYouLine tone="gain"><Amount sign="+" n={vp} unit="VP" />, held while every city has a camp ({wild!.with_camp} of {wild!.cities} do now) and lost the turn one is cleared. Call again to raise another camp in every city.</ForYouLine>
            : kind === 'RetireVeteran'
              ? <ForYouLine tone="gain">{progress && progress.after >= progress.target
                ? <><Amount sign="+" n={vp} unit="VP" />, kept for good: this is the last veteran owed.</>
                : <>{progress ? progress.target - progress.after : 0} more to retire for <Amount sign="+" n={vp} unit="VP" />, kept for good once the last is retired.</>}</ForYouLine>
              : <ForYouLine tone="gain"><Amount sign="+" n={vp} unit="VP" />, kept for good. It is one action, and cannot be repeated.</ForYouLine>}
          {kind === 'Swarm' && <>
            <ForYouLine tone="cost">{rows.length} {rows.length === 1 ? 'city shrinks' : 'cities shrink'}: their yields and city strength fall with population{tiles ? `, and ${tiles} ${tiles === 1 ? 'tile' : 'tiles'} of territory return to the map` : ''}.</ForYouLine>
            <ForYouLine tone="cost">Rivals will see a thin empire.</ForYouLine>
          </>}
          {kind === 'SakokuReopen' && <ForYouLine tone="gain">You can declare war and sign Alliances again, and technologies others know get cheaper for you again.</ForYouLine>}
          {kind === 'Broadcast' && <ForYouLine tone="cost">Every rival learns {broadcast!.techs_each} technologies without paying for them: {receivers.length} {receivers.length === 1 ? 'empire' : 'empires'} are stronger for it.</ForYouLine>}
          {kind === 'CallTheWild' && <>
            <ForYouLine tone="gain">Barbarians don't attack the Celts.</ForYouLine>
            <ForYouLine tone="cost">The tile a camp stands on, and any tile a barbarian unit holds, gives no yield.</ForYouLine>
            <ForYouLine tone="cost">A rival that clears a camp takes the usual Gold and ends your VP. You get no Gold for clearing the camps you raised, and a city founded later has none until you call again.</ForYouLine>
          </>}
          {kind === 'RetireVeteran' && retiree && <ForYouLine tone="cost">Your {retireName} (level {entry?.level}, {retiree.hp} HP) leaves the army for good. It cannot be brought back, and the army is weaker by that veteran.</ForYouLine>}
          <ForYouLine tone="note">Ordered now, it resolves when the turn ends. Cancel it {kind === 'RetireVeteran' ? 'from the unit panel' : 'from the race panel'} until then.</ForYouLine>
        </ul></div>
      </div>
    </div>
    {blocker && <p className="goal-why" role="alert">{blocker}</p>}
    <div className="end-turn-actions">
      <button ref={first} className={`btn primary${kind === 'Swarm' ? ' spends' : ''}`} disabled={!!blocker} aria-label={buttonLabel} aria-describedby={kind === 'Swarm' ? chip : undefined} onClick={confirm}>
        <span>{buttonLabel}{!touch && <> <Kbd>Return</Kbd></>}</span>
        {kind === 'Swarm' && totals && <CostChip id={chip} spends={[{ unit: 'population', amount: swarm!.population, from: totals.before, to: totals.after }]} />}
      </button>
      <button className="btn" aria-label="Cancel" onClick={close}>Cancel{!touch && <> <Kbd>Esc</Kbd></>}</button>
    </div>
  </dialog>;
}
