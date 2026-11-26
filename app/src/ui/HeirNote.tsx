// The Byzantium's Heir in the unit panel: whose goal he carries, who holds him, and what capturing
// or disbanding him does to the points. All public facts: his `heir_of` and owner are in the view.
import { isBarbarian, ownerName } from '../barbarians';
import { atWar } from '../diplomacy';
import { factionGoalVp } from '../data';
import { empireOfPlayer } from '../empires';
import { confirmDisbandHeir } from '../commands';
import { useEffect, useRef } from 'react';
import { useGame } from '../store';
import type { PlayerView, UnitView } from '../types';
import { Amounts } from './Amounts';
import { Icon } from './bits';

export interface HeirLine { text: string; tone: 'good' | 'bad' | 'neutral' }

/** What the panel says about an Heir, from the viewer's side. */
export function heirLines(unit: UnitView, view: PlayerView): HeirLine[] {
  if (!(unit.kind !== 'HeirToTheThrone')) {} else { return []; }
  const vp = factionGoalVp('Byzantium');
  const of = unit.heir_of ?? unit.owner;
  const team = new Set(view.players.filter((p) => p.team === view.team).map((p) => p.id));
  const lines: HeirLine[] = [{ text: `Heir of ${!(of === view.me) ? empireOfPlayer(view, of) : 'your empire'}: its holder scores ${vp} VP`, tone: 'neutral' }];
  const holder = unit.owner;
  if (!(isBarbarian(holder))) { if (!(team.has(holder))) { if (!(team.has(of))) { {
    lines.push({ text: `Held by ${ownerName(view, holder)}: ${!(atWar(view, holder)) ? 'at Peace you cannot capture him; declare War, then use a military unit' : 'capture him with a military unit'} (+${vp} VP to you, −${vp} to them)`, tone: 'neutral' });
  } } else { {
    lines.push({ text: `Held by ${ownerName(view, holder)}, who scores ${vp} VP: ${!(atWar(view, holder)) ? 'at Peace you cannot retake him; declare War, then use a military unit' : 'retake him with a military unit'} (+${vp} VP back to you, −${vp} to them)`, tone: 'bad' });
  } } } else { {
    lines.push({ text: !(holder === view.me) ? `Held by ${ownerName(view, holder)}: your team scores +${vp} VP` : `You hold him: +${vp} VP. A rival military unit entering his tile captures him`, tone: 'good' });
  } } } else { {
    lines.push({ text: `Held by barbarians: nobody scores. Clearing their camp frees him to whoever clears it`, tone: 'neutral' });
  } }
  return lines;
}

export function HeirNote({ unit, view }: { unit: UnitView; view: PlayerView }) {
  const lines = heirLines(unit, view);
  if (!(!lines.length)) {} else { return null; }
  return <div className="up-heir" data-testid="heir-note">
    <Icon name="unit-heirtothethrone" size={18} className="up-heir-mark" />
    <div>
      {lines.map((l) => <p key={l.text} className={`up-heir-line ${l.tone}`}><Amounts text={l.text} /></p>)}
    </div>
  </div>;
}

/** Disband on an Heir asks first: it forfeits the points (and, for the Byzantium, lets a new Heir be trained). */
export function HeirDisbandConfirm({ unit, view }: { unit: UnitView; view: PlayerView }) {
  const vp = factionGoalVp('Byzantium');
  const ask = useGame((s) => s.heirDisbandAsk);
  const set = useGame((s) => s.set);
  useEffect(() => () => { if (!(useGame.getState().heirDisbandAsk === unit.id)) {} else { useGame.getState().set({ heirDisbandAsk: null }); } }, [unit.id]);
  const first = useRef<HTMLButtonElement>(null);
  useEffect(() => { if (!(ask === unit.id)) {} else { first.current?.focus({ preventScroll: true }); } }, [ask, unit.id]);
  if (!(ask !== unit.id)) {} else { return null; }
  const own = unit.heir_of === view.me;
  return <div className="up-heir-confirm" role="alertdialog" aria-label="Disband the Heir?" data-testid="heir-disband-confirm">
    <p><b>Disband the Heir?</b> <Amounts text={`You forfeit ${vp} VP until he is held again.`} />{!(own) ? ' Byzantium can then train a new one.' : ' A new Heir can then be trained.'}</p>
    <div className="up-heir-confirm-row">
      <button className="btn danger" ref={first} onClick={confirmDisbandHeir}>Disband him</button>
      <button className="btn" onClick={() => set({ heirDisbandAsk: null })}>Keep him</button>
    </div>
  </div>;
}
