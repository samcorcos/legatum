import type { PlayerView } from '../types';
import type { VpModel, VpRow } from '../vpModel';
import { empireOfPlayer, empireTitle } from '../empires';
import { CounterDelta } from '../celebrate/MomentLane';
import { PortraitRoundel } from './premium';
import { TreatyEmblem } from './ringMedallion';
import { bordersOpen } from '../deals';
import { empireVars } from './vpParts';
import { Icon } from './bits';
import { CapitalChip, ClosedChip, DecadenceChip, LeadChip, TombChip, type RowRelation } from './VpHoldings';
import './rivals.css';

/** An army this many times yours or more reads red (cf. Civ 6's rival list: a threat is visible at a glance). */
export const STRONG_ARMY_RATIO = 1.5;

export interface RivalStats { gold: number; army: number; cities: number; science: number }

/** A table number: "1,396", and "23k" from 10,000 up so five digits never overrun a narrow column. */
export function shortNumber(n: number): string {
  return Math.abs(n) >= 10_000 ? `${Math.round(n / 1000)}k` : exact(n);
}
const exact = (n: number) => n.toLocaleString('en-US');

/** Portraits a row shows before the rest fold into a "+N" roundel, so a big team never squeezes its name to one letter a line. */
export const MAX_CRESTS = 2;
const crests = <T,>(members: T[]) => members.length > MAX_CRESTS ? members.slice(0, MAX_CRESTS - 1) : members;

/** " · AI 2" tells twin rows apart, so a wrap may fall before the dot but never strands the number. */
const keepSuffix = (name: string) => name.replace(/ · AI (\d+)/g, ' ·\u00a0AI\u00a0$1');

/** A team's public numbers: its members' Gold, army, cities and Science per turn added up. */
export function teamStats(view: PlayerView, row: VpRow): RivalStats {
  const members = view.players.filter((p) => row.members.some((m) => m.id === p.id));
  const sum = (f: (p: (typeof members)[number]) => number | undefined) => members.reduce((n, p) => n + (p.alive ? f(p) ?? 0 : 0), 0);
  return { gold: sum((p) => p.gold), army: sum((p) => p.army), cities: sum((p) => p.cities), science: sum((p) => p.science) };
}

/**
 * The rivals table (user, 2026-11-25, after Civ 6's rival list): every empire at a glance, ranked by VP, with its
 * Peace or War and its public Gold, army, cities and Science per turn. An empire you haven't met shows only its VP.
 */
export function RivalsTable({ view, model, relationOf, onOpen }: {
  view: PlayerView;
  model: VpModel;
  relationOf: (row: VpRow) => RowRelation | null;
  onOpen: (team: number) => void;
}) {
  const mine = model.rows.find((r) => r.isMine);
  const myArmy = mine ? teamStats(view, mine).army : 0;
  return (
    <div className="rivals" role="table" aria-label="Rivals">
      <div className="rv-grid rv-head" role="row">
        <span role="columnheader">Empire</span>
        <span role="columnheader" title="Victory points">VP</span>
        <span role="columnheader" title="Gold in the treasury" aria-label="Gold" className="rv-x"><Icon name="yield-gold" size={13} /></span>
        <span role="columnheader" title="Army points" aria-label="Army points" className="rv-x"><Icon name="title-strongest-army" size={13} /></span>
        <span role="columnheader" title="Cities" aria-label="Cities" className="rv-x"><Icon name="city" size={13} /></span>
        <span role="columnheader" title="Science per turn" aria-label="Science per turn" className="rv-x"><Icon name="yield-science" size={13} /></span>
      </div>
      {model.rows.map((row) => {
        const s = teamStats(view, row);
        const rel = row.isMine ? null : relationOf(row);
        // A War names the enemy even before contact (`unmetTeam`), and so does your own declaration queued this turn.
        const unmet = row.unmet && !rel?.declared;
        const strong = !row.isMine && !unmet && myArmy >= 0 && s.army > 0 && s.army >= myArmy * STRONG_ARMY_RATIO;
        const label = row.isMine ? (row.members.length > 1 ? 'Your team' : 'You') : unmet ? 'Unknown' : row.unmet ? empireTitle(view, row.team) : row.label;
        const vp = <span className="rv-num rv-vp" role="cell" data-celebration-counter={row.isMine ? 'vp' : undefined}><b>{row.vp}</b><small>/{row.target}</small>{row.isMine && <CounterDelta counter="vp" />}</span>;
        const cells = unmet
          ? <span className="rv-hidden rv-x" role="cell">Not met yet</span>
          : <>
            <span className="rv-num rv-x" role="cell" title={`Gold in the treasury: ${exact(s.gold)}`}>{shortNumber(s.gold)}</span>
            <span className={`rv-num rv-x${strong ? ' strong' : ''}`} role="cell" title={strong ? `Army points ${exact(s.army)}: ${STRONG_ARMY_RATIO} times yours or more` : `Army points ${exact(s.army)}`}>{shortNumber(s.army)}</span>
            <span className="rv-num rv-x" role="cell" title={`Cities: ${exact(s.cities)}`}>{shortNumber(s.cities)}</span>
            <span className="rv-num rv-x" role="cell" title={`Science per turn: ${exact(s.science)}`}>{shortNumber(s.science)}</span>
          </>;
        const who = <span className="rv-who" role="cell">
          <span className="rv-crests">
            {unmet ? <span className="rv-unknown" aria-hidden="true">?</span>
              : crests(row.members).map((m) => <PortraitRoundel key={m.id} faction={m.faction} size={22} color={m.ink} />)}
            {!unmet && row.members.length > MAX_CRESTS && <span className="rv-more" title={row.members.map((m) => empireOfPlayer(view, m.id)).join(', ')}>+{row.members.length - MAX_CRESTS + 1}</span>}
          </span>
          <span className="rv-name-col">
            <span className="rv-name">{keepSuffix(label)}</span>
            {rel && !unmet && <span className={`rv-rel ${rel.kind}${rel.declared ? ' declared' : ''}${rel.isNew ? ' new' : ''}`} title={rel.title} data-relation={rel.text}>
              <TreatyEmblem kind={rel.kind === 'war' ? 'War' : rel.kind === 'alliance' ? 'Alliance' : 'Peace'} size={12} /><span className="rv-rel-text">{rel.text}</span>
            </span>}
          </span>
        </span>;
        // Public chips get their own line across the row, so a long one never runs into the numbers.
        const chips = !unmet && <span className="rv-status">
          {!row.isMine && bordersOpen(view, row.team) && <span className="rv-borders" data-testid="rivals-open-borders"
            title="Open Borders: your units may enter each other’s land."><TreatyEmblem kind="OpenBorders" size={12} />Open Borders</span>}
          <ClosedChip members={row.members} />
          <TombChip members={row.members} />
          <LeadChip members={row.members} rules={model.raceRules.rules} />
          <DecadenceChip members={row.members} rules={model.raceRules.rules} />
          {!row.capitalStands && <CapitalChip row={row} nameOf={(e) => e.isMe ? 'You' : empireOfPlayer(view, e.id)} />}
        </span>;
        const bar = <span className="rv-bar" aria-hidden="true"><i style={{ width: `${Math.min(100, (row.vp / Math.max(1, row.target)) * 100)}%` }} /></span>;
        const cls = `rv-grid rv-row${row.isMine ? ' mine' : ''}${unmet ? ' unmet' : ''}${rel?.kind === 'war' ? ' at-war' : ''}${row.hold ? ' holding' : ''}`;
        const aria = `${label}: ${row.vp} of ${row.target} VP${rel && !unmet ? `, ${rel.text}` : ''}${unmet ? '' : `, Gold ${s.gold}, army points ${s.army}, ${s.cities} cities, Science ${s.science} per turn`}`;
        const style = unmet ? undefined : empireVars(row.members[0]);
        // A met rival's row opens Diplomacy with it; your own row and an unmet one are not buttons.
        return row.isMine || unmet
          ? <div key={row.team} className={cls} style={style} role="row" aria-label={aria}>{who}{vp}{cells}{chips}{bar}</div>
          : <button key={row.team} type="button" className={cls} style={style} role="row" aria-label={`${aria}. Open Diplomacy`} title={`${row.members.filter((m) => m.aiLevel).map((m) => `${row.members.length > 1 ? `${empireOfPlayer(view, m.id)}: ` : ''}AI · ${m.aiLevel}. `).join('')}Open Diplomacy`} onClick={() => onOpen(row.team)}>{who}{vp}{cells}{chips}{bar}</button>;
      })}
    </div>
  );
}
