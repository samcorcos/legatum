import { GoalActionRow } from './GoalActions';
import { useGame } from '../store';
import { Amounts } from './Amounts';
import { FACTION_GOAL, factionGoalVp, factionGoalRule, factionGoalTerms } from '../data';
import type { PlayerView } from '../types';
import { Tip } from './Tip';
import { cappedGoalLabel } from './goalText';

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

/** Action goals stay in sight; passive goal details may be disclosed. */
export function SidebarFactionGoal({ view }: { view: PlayerView }) {
  const me = view.players.find(p => p.id === view.me), p = view.faction_goal;
  if (!me || !p || view.finished) return null;
  const goal = FACTION_GOAL[me.faction];
  const action = !!(view.actions?.swarm || view.actions?.sakoku || view.actions?.broadcast || view.actions?.wild || view.actions?.retire);
  if (action) return <section className="sidebar-goal sidebar-goal-action">
    <GoalActionRow />
  </section>;
  return <details className="sidebar-goal">
    <summary><span>{goal.name}</span><span>{p.done ? (p.live ? 'Held' : 'Scored') : `${Math.min(p.value, p.target)}/${p.target}`} · <Amounts text={`+${factionGoalVp(me.faction)} VP`} /></span></summary>
    <p>{factionGoalRule(me.faction)}. {cappedGoalLabel(p.label)}</p>
    <button className="link" onClick={() => useGame.getState().set({ vpExpanded: true })}>Goal in the score ledger</button>
  </details>;
}
