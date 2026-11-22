import { GoalActionRow } from './GoalActions';
import { useGame } from '../store';
import { Amounts } from './Amounts';
import { FACTION_GOAL, factionGoalVp, factionGoalRule, factionGoalTerms } from '../data';
import type { PlayerView } from '../types';
import { Tip } from './Tip';
import { cappedGoalLabel } from './goalText';
import { RingMedallion } from './ringMedallion';
import { VpSeal } from './vpParts';
import { goalCities, COLONIA_SITE_RULE, tombChip, tombShort } from '../goalCities';
import { Icon } from './bits';
import { coloniaFounder } from '../goalDecision';
import { empireOfPlayer } from '../empires';

const number = (n: number) => n.toLocaleString('en-US');

/** A small This Turn reminder when the faction goal is close or already held. An available goal action already has its own row above, so it gets no second line. */
export function FactionGoalProgressLine({ view }: { view: PlayerView }) {
  const p = view.faction_goal;
  const me = view.players.find((x) => x.id === view.me);
  if (!p || !me) return null;
  const actions = view.actions;
  const achievable = !p.done && (me.faction === 'Athens' ? !!actions?.broadcast && actions.broadcast.blocker === null
      : me.faction === 'Celts' ? !!actions?.wild && actions.wild.blocker === null
        : me.faction === 'Sparta' ? !!actions?.retire?.veterans.some((v) => v.blocker === null)
          : false);
  if (achievable || (!p.done && p.value / Math.max(1, p.target) < 0.75)) return null;
  const name = FACTION_GOAL[me.faction].name;
  const label = cappedGoalLabel(p.label);
  const progressLabel = me.faction === 'Byzantium'
    ? label.replace(/^The Heir (?:to the Throne )?is yours$/, 'Yours').replace(/^The Heir (?:to the Throne )?is not trained$/, 'Not trained')
    : label;
  return <Tip className="faction-goal-progress" content={<><b>{name}</b><p>{factionGoalRule(me.faction)}.</p><p>{progressLabel} · {factionGoalTerms(me.faction)}</p></>}>
    <span><b>{name}</b> · {progressLabel}</span>
    <span className="faction-goal-meter" role="progressbar" aria-label={`${name}: ${label}`} aria-valuemin={0} aria-valuemax={p.target} aria-valuenow={Math.min(p.value, p.target)}><i style={{ width: `${Math.min(100, Math.max(0, p.value / Math.max(1, p.target) * 100))}%` }} /></span>
  </Tip>;
}

/** The one-line treasury target shown beside the Gold readout for Switzerland. */
export function HoardReadout({ view, gold }: { view: PlayerView; gold?: number }) {
  const p = view.faction_goal;
  const me = view.players.find((x) => x.id === view.me);
  if (!p || me?.faction !== 'Switzerland') return null;
  return <small className="gold-hoard-readout" data-testid="gold-hoard-readout">
    Reserve {number(gold ?? p.value)} / {number(p.target)}
  </small>;
}

/** Egypt's own tomb work beside the Gold readout while it is on: every city is stopped, so the count stays in sight. */
export function TombReadout({ view }: { view: PlayerView }) {
  const t = view.actions?.tomb;
  if (!t?.working) return null;
  const st = { working: true, turns: t.turns, target: t.target, left: Math.max(0, t.target - t.turns) };
  return <Tip className="stat tomb-stat" content={<><p><b>{tombShort(st)}</b></p><p>None of your cities builds or grows while the work is on. Gold and Science still come in.</p><p className="muted">Pause it from the Victory race (G). The count is kept.</p></>}>
    <span className="tomb-readout" data-testid="tomb-readout" style={{ ['--tomb-done' as string]: `${Math.round(100 * st.turns / Math.max(1, st.target))}%` }}><Icon name="goal-royal-tomb" size={16} /><span className="tomb-long">{tombShort(st)}</span><b className="tomb-short">{tombChip(st)}</b></span>
  </Tip>;
}

/** Action goals stay in sight; passive goal details may be disclosed. */
export function SidebarFactionGoal({ view }: { view: PlayerView }) {
  const me = view.players.find(p => p.id === view.me), p = view.faction_goal;
  if (!me || !p || view.finished) return null;
  const goal = FACTION_GOAL[me.faction];
  if (me.faction === 'Rome') return <section className="sidebar-goal sidebar-goal-action"><ColoniaGoalRow view={view} /></section>;
  const action = !!(view.actions?.swarm || view.actions?.sakoku || view.actions?.broadcast || view.actions?.wild || view.actions?.retire || view.actions?.tomb);
  if (action) return <section className="sidebar-goal sidebar-goal-action">
    <GoalActionRow />
  </section>;
  return <details className="sidebar-goal">
    <summary><span>{goal.name}</span><span>{p.done ? (p.live ? 'Held' : 'Scored') : `${Math.min(p.value, p.target)}/${p.target}`} · <Amounts text={`+${factionGoalVp(me.faction)} VP`} /></span></summary>
    <p>{factionGoalRule(me.faction)}. {cappedGoalLabel(p.label)}</p>
    <button className="link" onClick={() => useGame.getState().set({ vpExpanded: true })}>Goal in the score ledger</button>
  </details>;
}

/**
 * Rome's Colonia Victrix in the sidebar, in its three states: the best veteran's level toward the
 * founding level (before), a veteran ready to found it (ready: the button selects him, and the
 * Settle lens rings the sites), and the city that is the Colonia (held, or held by a rival).
 */
function ColoniaGoalRow({ view }: { view: PlayerView }) {
  const p = view.faction_goal!, goal = FACTION_GOAL.Rome;
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const standing = goalCities(view).find((g) => g.kind === 'colonia' && g.of === view.me);
  const founder = coloniaFounder(view, orders);
  const ordered = !standing && view.actions?.colonia?.founders.some((u) => orders.some((o) => o.type === 'FoundCity' && o.unit === u));
  const state = standing ? (standing.holder === view.me ? 'held' : 'lost') : founder !== null || ordered ? 'ready' : 'before';
  const label = cappedGoalLabel(p.label);
  const selected = useGame((s) => s.selection);
  const founderSelected = founder !== null && selected?.kind === 'unit' && selected.id === founder;
  const tip = <><p>{factionGoalRule('Rome')}.</p><p className="muted">{factionGoalTerms('Rome')}.</p></>;
  // The short window's one line keeps the goal's name and reward: "Colonia Victrix · +2 VP: best melee veteran is level 2 of 3".
  const c = view.actions?.colonia, level = c?.level ?? 3;
  const lost = state === 'lost' ? `${empireOfPlayer(view, standing!.holder)} holds it in ${standing!.name}. Retake the city.` : null;
  const short = ordered ? 'founded when the turn ends'
    : state === 'held' ? `you hold it in ${standing!.name}` : lost ? lost
    : state === 'ready' ? `a level ${level} melee veteran is ready`
    : c && c.best > 0 ? `best melee veteran is level ${c.best} of ${level}` : 'no melee veteran yet';
  return <section className={`goal-action${state === 'held' ? ' ordered' : ''}`} aria-label={`${goal.name}: your faction goal`} data-goal-action="Colonia" data-colonia-state={state}>
    <Tip as="div" className="goal-action-head" content={tip}>
      <RingMedallion icon={goal.icon} tone={state === 'held' ? 'gilt' : 'hollow'} size={28} />
      <h3>{goal.name}</h3>
      <VpSeal vp={factionGoalVp('Rome')} />
    </Tip>
    <p className="goal-action-note goal-action-progress colonia-status" data-testid="colonia-progress">{ordered ? 'Ordered: your veteran founds it when the turn ends' : lost ?? label}</p>
    <p className="goal-action-note colonia-compact" data-testid="colonia-compact">{goal.name} · +{factionGoalVp('Rome')} VP: {short}</p>
    {state === 'before' && <span className="faction-goal-meter" role="progressbar" aria-label={`${goal.name}: ${label}`} aria-valuemin={0} aria-valuemax={p.target} aria-valuenow={Math.min(p.value, p.target)}><i style={{ width: `${Math.min(100, 100 * p.value / Math.max(1, p.target))}%` }} /></span>}
    {state === 'ready' && founder !== null && !founderSelected && <button className="btn goal-action-btn" onClick={() => useGame.getState().select({ kind: 'unit', id: founder }, true)}>Select the veteran</button>}
    {(state === 'held' || state === 'lost') && standing && <button className="link" onClick={() => useGame.getState().jumpTo(standing.at)}>Show on map</button>}
    {state === 'before' && <p className="goal-action-why colonia-why">Promote a melee unit to level {view.actions?.colonia?.level ?? 3}. {COLONIA_SITE_RULE}</p>}
  </section>;
}
