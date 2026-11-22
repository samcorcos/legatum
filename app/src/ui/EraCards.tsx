import { useState, type KeyboardEvent, type MouseEvent } from 'react';
import { TECH_NAME } from '../data';
import type { Faction, PlayerView, TechId, TechOption } from '../types';
import { Amount } from './Amounts';
import { BreakdownTable, Icon } from './bits';
import { arrivalText, costParts, diffusionLine, eraNumeral, eraRows, eraFirst, turnsText, type EraRow, type QueueSlot } from './eraResearch';
import { Tip } from './Tip';
import { techCardUnlocks } from './techCardUnlocks';
import { BenefitLedger } from './BenefitLedger';
import { empireOfPlayer } from '../empires';

interface CardProps {
  view: PlayerView;
  tech: TechOption;
  faction: Faction;
  queueIndex: number;
  slot: QueueSlot | undefined;
  stored: number;
  selected: boolean;
  noScience: boolean;
  onDetails(tech: TechId): void;
  onResearchNow(tech: TechId): void;
  onQueue(tech: TechId): void;
  onRemove(tech: TechId): void;
  /** The tech whose pick animation is playing (a short glow on its card). */
  picked?: boolean;
}

/** One current-era technology: the only kind of card the strict gate lets you choose. */
export function EraCard({ view, tech: option, faction, queueIndex, slot, stored, selected, noScience, onDetails, onResearchNow, onQueue, onRemove, picked }: CardProps) {
  const t = option.tech;
  const known = view.research.known.includes(t);
  const researching = queueIndex === 0 && !known;
  const queued = queueIndex >= 0 && !known;
  const waiting = queued && !researching;
  const turns = slot?.turns ?? option.turns;
  const locked = view.finished;
  const parts = costParts(option.cost);
  const discount = [diffusionLine(parts)].filter((x): x is string => !!x);
  const unlocks = techCardUnlocks(view.research, t, faction);
  const cost = option.cost.total;
  const progress = researching ? Math.min(stored, cost) : 0;
  const state = known ? 'Researched' : researching ? 'Researching' : queued ? `Queue ${queueIndex + 1}` : 'Available';
  const when = known ? 'Researched' : queued && slot ? (slot.arrivalTurn === null ? arrivalText(slot, noScience) : `Turn ${slot.arrivalTurn}`) : option.turns === null ? (noScience ? 'No Science yet' : 'Needs Science') : `Alone: ${turnsText(option.turns)}`;
  /** The whole card is the target (Civ style): click researches now, Shift+click queues. Buttons inside keep their own meaning. */
  const pick = (shift: boolean) => { if (known || locked) return; if (shift) onQueue(t); else onResearchNow(t); };
  const onClick = (e: MouseEvent<HTMLElement>) => {
    if (!e.currentTarget.contains(e.target as Node) || (e.target as HTMLElement).closest('button, a, summary, input, select')) return;
    pick(e.shiftKey);
  };
  const onKeyDown = (e: KeyboardEvent<HTMLElement>) => {
    if (e.target !== e.currentTarget || e.key !== 'Enter') return;
    e.preventDefault();
    pick(e.shiftKey);
  };
  return <article className={`era-card ${known ? 'known' : researching ? 'researching' : queued ? 'queued' : 'available'}${selected ? ' chosen' : ''}${picked ? ' picked' : ''}`}
    data-tech={t} data-state={state.toLowerCase().replace(/ \d+$/, '')} aria-label={`${TECH_NAME[t]}. ${state}. ${when}.`}
    tabIndex={known ? undefined : 0} data-enter-activates={known ? undefined : ''} onClick={known ? undefined : onClick} onKeyDown={known ? undefined : onKeyDown}
    // Shift+click must not select text across the cards.
    onMouseDown={(e) => { if (e.shiftKey) e.preventDefault(); }}>
    {waiting && <span className="era-queue-tag" data-testid="research-order-badge" data-position={queueIndex + 1}>
      <b>Queued #{queueIndex + 1}</b>
      <button className="era-queue-x" disabled={locked} aria-label={`Remove ${TECH_NAME[t]} from the queue`} title="Remove from the queue" onClick={() => onRemove(t)}><Icon name="ui-cross" size={11} /></button>
    </span>}
    <header className="era-card-head">
      <span className="era-seal" aria-hidden="true">{known ? <Icon name="ui-check" size={13} /> : eraNumeral(option.era)}</span>
      <h4 className={`era-card-name${TECH_NAME[t].split(' ').some((w) => w.length > 12) ? ' long-word' : ''}`}><span className="era-card-title">{TECH_NAME[t]}</span></h4>
      <button type="button" className="era-details-chevron" aria-expanded={selected} aria-label={`${selected ? 'Hide' : 'Show'} details of ${TECH_NAME[t]}`} title={selected ? 'Hide details' : 'Details: cost and what it unlocks'} onClick={() => onDetails(t)}>
        <Icon name={selected ? 'ui-chevron-up' : 'ui-chevron-down'} size={12} />
      </button>
    </header>
    <BenefitLedger entries={unlocks} faction={faction} />
    {!known && <>
      <div className="era-cost">
        <Tip content={<BreakdownTable b={option.cost} title="Research cost" unit="Science" />} summary={`Research cost: ${cost} Science`}>
          <span className="era-cost-main" data-testid="era-cost">
            {discount.length > 0 && <s className="era-cost-base" title="Listed price before the discounts">{parts.base}</s>}
            <Amount n={researching ? `${progress}/${cost}` : cost} unit="Science" />
          </span>
        </Tip>
        <span className="era-when" data-testid="era-when">{when}</span>
      </div>
      <div className={`era-cost-note${discount.length ? '' : ' none'}`} data-testid="era-discounts">{discount.length ? discount.join(' · ') : 'No discounts yet'}</div>
    </>}
    {!known && <footer className="era-card-foot">
      {researching ? <div className="era-research-status" data-testid="era-researching">
          <span className="era-now">Researching · {turns === null ? (noScience ? 'No Science yet' : 'needs Science') : turnsText(turns)}</span>
          <span className="era-progress" role="progressbar" aria-label={`${TECH_NAME[t]} research progress`} aria-valuemin={0} aria-valuemax={cost} aria-valuenow={progress}><span style={{ width: `${cost ? progress * 100 / cost : 0}%` }} /></span>
        </div>
        : <>
          <button className={queued ? 'btn quiet' : 'btn'} disabled={locked} data-testid="research-now" title="Research now · or click anywhere on the card (Enter on a focused card)" onClick={() => onResearchNow(t)}>Research now</button>
          {!queued && <button className="btn quiet" disabled={locked} data-testid="queue-add" aria-label={`Add ${TECH_NAME[t]} to the queue`} title="Add to the end of the queue · or Shift+click the card (Shift+Enter on a focused card)" onClick={() => onQueue(t)}>+ Queue</button>}
        </>}
    </footer>}
  </article>;
}

/** The eras that are not current: past ones done, later ones named with their tech count, each opening a read-only peek. */
export function EraRows({ view }: { view: PlayerView }) {
  const rows = eraRows(view.research).filter((row) => row.state !== 'current');
  const [open, setOpen] = useState<number | null>(null);
  const first = (row: EraRow) => {
    const who = eraFirst(view.research, row.era);
    // The empire's name (its faction), never a seat label such as "AI 6".
    const name = who === null ? null : who === view.me ? 'you' : view.players.some((p) => p.id === who) ? empireOfPlayer(view, who) : null;
    return name ? `first: ${name}` : null;
  };
  if (!rows.length) return null;
  return <section className="era-rows" aria-label="Eras">
    <ol>{rows.map((row) => {
      const isOpen = open === row.era;
      const note = first(row);
      return <li key={row.era} className={`era-row ${row.state}`} data-era={row.era} data-state={row.state}>
        <button className="era-row-main" aria-expanded={isOpen} aria-controls={`era-peek-${row.era}`} onClick={() => setOpen(isOpen ? null : row.era)}>
          <span className="era-row-numeral" aria-hidden="true">{row.numeral}</span>
          <span className="era-row-name">{row.name}</span>
          <span className="era-row-count">{row.state === 'past' ? <><Icon name="ui-check" size={11} className="ui-mark" /> {row.known} of {row.total} known</> : <>{row.total} technologies · locked until the era before is finished</>}</span>
          {note && <span className="era-row-first">{note}</span>}
          <span className="era-row-chevron" aria-hidden="true"><Icon name={isOpen ? 'ui-chevron-up' : 'ui-chevron-down'} size={11} className="ui-mark" /></span>
        </button>
        {isOpen && <ul id={`era-peek-${row.era}`} className="era-peek" aria-label={`${row.name} technologies`}>{row.techs.map((t) => <li key={t}>{TECH_NAME[t]}</li>)}</ul>}
      </li>;
    })}</ol>
  </section>;
}
