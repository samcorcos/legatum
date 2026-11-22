import { useEffect, useId, useLayoutEffect, useRef, useState, type Ref } from 'react';
import { empireOfPlayer } from '../empires';
import { useGame } from '../store';
import { feel } from '../feel';
import type { Faction, PlayerView, TechId } from '../types';
import { ERA_COUNT, TECH_NAME } from '../data';
import { effectiveResearchQueue } from '../orders';
import { firstScienceForecast } from '../research';
import { revealedByTech } from '../resources';
import { resourceArtUrl } from '../icons';
import { ResearchQueue, editQueue } from './ResearchQueue';
import { flyToQueue } from './flyChip';
import { BreakdownTable, Icon, Tip, YieldIcon } from './index';
import { closeTips } from './Tip';
import { Amount, Amounts, YieldAmounts } from './Amounts';
import { EraCard, EraRows } from './EraCards';
import { UnlockDetail } from './TechDetail';
import { EraObjectivesResearch } from './EraObjectives';
import { techCardUnlocks } from './techCardUnlocks';
import { appendToQueue, costParts, currentEraTechs, diffusionLine, eraFirst, eraRows, eraName, eraNumeral, queueSchedule, researchCatalog, researchFirst, type QueueSlot } from './eraResearch';
import './research.css';
import { pendingSpend } from './productionPurchases';

/** The current era as the player's own place in the timeline: numeral, name, N of 5 pips, who was first in. */
function EraBadge({ view }: { view: PlayerView }) {
  const r = view.research;
  const who = eraFirst(r, r.era);
  const first = who === null ? null : who === view.me ? 'You were first into this era' : `${empireOfPlayer(view, who)} was first into this era`;
  const rows = eraRows(r);
  return <div className="era-badge" data-testid="era-badge" aria-label={`${r.era_name} era, ${r.era_known} of ${r.era_total} technologies known`}>
    <span className="era-badge-numeral" aria-hidden="true">{eraNumeral(r.era)}</span>
    <span className="era-badge-text">
      <b className="era-badge-name">{eraName(r.era)} era</b>
      <span className="era-badge-count" data-testid="era-progress">{r.era_known} of {r.era_total} known</span>
      <span className="era-track" aria-hidden="true" title={rows.map((row) => `${row.name}${row.state === 'past' ? ' (done)' : row.state === 'current' ? ' (now)' : ''}`).join(' · ')}>{rows.map((row) => <i key={row.era} className={row.state}>{row.numeral}</i>)}</span>
    </span>
    <span className="era-pips" aria-hidden="true">{Array.from({ length: r.era_total }, (_, i) => <i key={i} className={i < r.era_known ? 'on' : ''} />)}</span>
    {first && <span className="era-badge-first">{first}</span>}
  </div>;
}

export function TechPanel() {
  const open = useGame((s) => s.techOpen);
  const view = useGame((s) => s.views ? s.views[s.seat] : null);
  const orders = useGame((s) => s.orders[s.seat]);
  const errors = useGame((s) => s.errors[s.seat]);
  const set = useGame((s) => s.set);
  const editOrders = useGame((s) => s.editOrders);
  const dialog = useRef<HTMLDialogElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const detail = useRef<HTMLElement>(null);
  const closing = useRef(false);
  const titleId = useId();
  const [inspected, inspect] = useState<TechId | null>(null);
  // The tech just picked: its card glows and a chip flies into the queue strip (about 0.4 s, never moves layout).
  const [picked, setPicked] = useState<{ tech: TechId; n: number } | null>(null);
  const pickCount = useRef(0);
  const visible = open && !!view;
  useEffect(() => { inspect(null); }, [visible, view?.me]);
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!visible || !el) return;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    closeTips();
    el.showModal();
    closeButton.current?.focus({ preventScroll: true });
    return () => {
      // Cleanup is not a user dismissal (including StrictMode's effect replay).
      closing.current = true;
      el.close();
      closing.current = false;
      if (opener?.isConnected) opener.focus({ preventScroll: true });
    };
  }, [visible]);
  // Clearing a queue can remove the focused control and leave focus on <body>.
  // Keep Escape available even when focus has fallen outside the dialog.
  useEffect(() => {
    if (!visible) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || e.defaultPrevented || dialog.current?.contains(e.target as Node)) return;
      e.preventDefault();
      set({ techOpen: false });
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [visible, set]);
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!picked || !el) return;
    const card = el.querySelector<HTMLElement>(`.era-card[data-tech="${picked.tech}"]`);
    const slot = el.querySelector<HTMLElement>(`.research-queue li[data-tech="${picked.tech}"]`);
    // The pressed button is gone once the card changes state: keep keyboard focus on the card.
    if (card && (document.activeElement === document.body || card.contains(document.activeElement))) card.focus({ preventScroll: true });
    // The strip is one row that scrolls sideways when it is narrow: bring the slot into view (sideways only).
    const strip = slot?.closest<HTMLElement>('.research-queue-slots');
    if (slot && strip) {
      const left = slot.offsetLeft - strip.offsetLeft, right = left + slot.offsetWidth;
      if (right > strip.scrollLeft + strip.clientWidth) strip.scrollLeft = right - strip.clientWidth + 4;
      else if (left < strip.scrollLeft) strip.scrollLeft = Math.max(0, left - 4);
    }
    const cancel = card && slot ? flyToQueue(el, card, slot, TECH_NAME[picked.tech]) : undefined;
    const done = window.setTimeout(() => setPicked((p) => (p?.n === picked.n ? null : p)), 520);
    return () => { cancel?.(); window.clearTimeout(done); };
  }, [picked]);
  useEffect(() => {
    if (!inspected) return;
    detail.current?.scrollIntoView({ block: 'nearest', behavior: 'instant' });
    // A short window can't hold a tall card and its details at once: keep the card's name in view, details below.
    const scroller = detail.current?.closest<HTMLElement>('.research-scroll');
    const head = scroller?.querySelector<HTMLElement>(`.era-card[data-tech="${inspected}"]`);
    if (scroller && head && head.getBoundingClientRect().top < scroller.getBoundingClientRect().top) scroller.scrollTop -= scroller.getBoundingClientRect().top - head.getBoundingClientRect().top + 4;
  }, [inspected]);
  if (!visible || !view) return null;
  const r = view.research;
  const faction = view.players.find((p) => p.id === view.me)?.faction ?? 'India';
  const queue = effectiveResearchQueue(orders, r);
  const science = view.economy.science_per_turn.total;
  const conversionOrder = orders?.findLast((o) => o.type === 'SetScienceConversion');
  const conversion = conversionOrder?.type === 'SetScienceConversion' ? conversionOrder.to : r.science_conversion ?? 'Gold';
  const cityScience = view.economy.science_per_turn.parts.filter(([label]) => view.cities.some((c) => c.owner === view.me && c.name === label));
  const localForecast = cityScience.reduce((sum, [, amount]) => sum + Math.floor(amount / 2), 0);
  const chooseConversion = (to: 'Gold' | 'Food' | 'Production') => editOrders((current) => [
    ...current.filter((o) => o.type !== 'SetScienceConversion'),
    { type: 'SetScienceConversion', to },
  ], 'queue');
  const noScience = firstScienceForecast(view);
  const cards = currentEraTechs(r);
  // The techs this faction researches: a skipped one (it gives the faction nothing) is not counted.
  const catalog = researchCatalog(r);
  const knownCount = catalog.filter((t) => r.known.includes(t.tech)).length;
  const schedule = queueSchedule(r, queue, science, view.turn);
  const selected = inspected && catalog.some((t) => t.tech === inspected) ? inspected : null;
  const toggleDetails = (tech: TechId) => { feel.select(); inspect(selected === tech ? null : tech); };
  // editQueue commits the order, which carries its own feedback: a select first would swallow it (80 ms debounce).
  const announce = (tech: TechId) => setPicked({ tech, n: ++pickCount.current });
  const researchNow = (tech: TechId) => {
    if (view.finished || queue[0] === tech) return;
    editQueue(view, researchFirst(queue, tech));
    announce(tech);
  };
  const addToQueue = (tech: TechId) => {
    if (view.finished || queue.includes(tech)) return;
    editQueue(view, appendToQueue(queue, tech));
    announce(tech);
  };
  const remove = (tech: TechId) => editQueue(view, queue.filter((t) => t !== tech));

  return <dialog ref={dialog} className="modal tech-modal" aria-labelledby={titleId} data-dismissible="research"
    onCancel={(e) => { e.preventDefault(); set({ techOpen: false }); }}
    onClose={(e) => { if (!closing.current && !e.currentTarget.open) set({ techOpen: false }); }}
    onPointerDown={(e) => {
      if (e.target !== e.currentTarget) return;
      const rect = e.currentTarget.getBoundingClientRect();
      if (e.clientX < rect.left || e.clientX > rect.right || e.clientY < rect.top || e.clientY > rect.bottom) set({ techOpen: false });
    }}
    onKeyDown={(e) => {
      // Keep game shortcuts out of the modal, but retain native control keys.
      e.stopPropagation();
      if (e.key === 'Escape') { e.preventDefault(); set({ techOpen: false }); }
      if (e.key !== 'Tab') return;
      const stops = [...e.currentTarget.querySelectorAll<HTMLElement>('button:not(:disabled), select:not(:disabled), summary, [tabindex="0"]')]
        .filter((el) => !el.closest('[hidden], [inert]') && (!el.closest('details:not([open])') || (el.tagName === 'SUMMARY' && el.parentElement?.matches('details:not([open])'))));
      const first = stops[0], last = stops.at(-1);
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
    }}>
      <div className="modal-head">
        <div className="research-head-main"><h2 id={titleId}>Research</h2><div className="muted"><Amount n={science} unit="Science" /> per turn · <Amount n={r.progress} unit="Science" /> stored · {knownCount} of {catalog.length} technologies known</div>
          {noScience && <p className="research-no-science"><Amounts text={noScience} /></p>}</div>
        <EraBadge view={view} />
        <div className="research-head-actions"><button ref={closeButton} className="btn quiet" onClick={() => set({ techOpen: false })} aria-label="Close research">Close <kbd>Esc</kbd></button></div>
      </div>
      <ResearchQueue view={view} orders={orders} slotCount={cards.filter((c) => c.researchable && !r.known.includes(c.tech)).length} landed={picked?.tech} />
      <div className="research-scroll">
        {knownCount === catalog.length && <section className="science-conversion" aria-label="Science after research">
          <h3>After the last technology</h3>
          <p>Every technology is known. Half your Science now becomes Gold, Food or Production: choose which. You can change it every turn.</p>
          <div className="science-conversion-options" role="group" aria-label="What half your Science becomes">
            {(['Gold', 'Food', 'Production'] as const).map((to) => {
              const gain = to === 'Gold' ? Math.floor(science / 2) : localForecast;
              const chosen = conversion === to;
              return <button key={to} type="button" className={`science-choice${chosen ? ' chosen' : ''}`} data-conversion={to} aria-pressed={chosen} onClick={() => chooseConversion(to)}>
                <span className="science-choice-name"><YieldIcon k={to.toLowerCase() as 'gold' | 'food' | 'production'} size={20} /><b>{to}</b>{chosen && <span className="science-choice-mark"><Icon name="ui-check" size={12} /> Chosen</span>}</span>
                <span className="science-choice-gain"><Amount sign="+" n={gain} unit={to} /> per turn</span>
                <small>{to === 'Gold' ? 'Half of your empire’s Science goes to the treasury.' : `Each city turns half its own Science into ${to}: ${cityScience.map(([name, amount]) => `${name} +${Math.floor(amount / 2)}`).join(', ')}.`}</small>
              </button>;
            })}
          </div>
        </section>}
        <section className="era-current" aria-label={`${eraName(r.era)} era technologies`}>
          <h3 className="era-heading">{eraName(r.era)} era <span className="muted">· only these can be researched now{r.era < ERA_COUNT - 1 ? '; finish them all to enter the next era' : ''}</span></h3>
          <div className="era-cards" data-count={cards.length}>
            {cards.map((option) => {
              const queueIndex = queue.indexOf(option.tech);
              return <EraCard key={option.tech} view={view} tech={option} faction={faction} queueIndex={queueIndex} slot={schedule[queueIndex]}
                stored={r.progress} selected={selected === option.tech} noScience={!!noScience}
                picked={picked?.tech === option.tech} onDetails={toggleDetails} onResearchNow={researchNow} onQueue={addToQueue} onRemove={remove} />;
            })}
          </div>
        </section>
        {selected && <TechDetails ref={detail} view={view} tech={selected} faction={faction} slot={schedule[queue.indexOf(selected)]} gold={pendingSpend(view, orders ?? [], errors ?? []).gold} />}
        <EraObjectivesResearch view={view} />
        <EraRows view={view} />
      </div>
      <div className="modal-foot muted">Forecasts use today's Science, prices and discounts; other empires and city growth can change them. Stored Science carries over, with at most one technology finished per turn.</div>
  </dialog>;
}

/** The selected technology in full: cost breakdown, what it unlocks and what building it first costs. */
function TechDetails({ ref, view, tech, faction, slot, gold }: { ref?: Ref<HTMLElement>; view: PlayerView; tech: TechId; faction: Faction; slot: QueueSlot | undefined; gold: number }) {
  const r = view.research;
  const option = r.catalog.find((t) => t.tech === tech)!;
  const known = r.known.includes(tech);
  const unlock = r.unlocks.find((u) => u.tech === tech);
  const effects = techCardUnlocks(r, tech, faction).filter((entry) => entry.kind === 'Effect').map((entry) => entry.label);
  const parts = costParts(option.cost);
  const turns = slot?.turns ?? option.turns;
  return <section ref={ref} className="research-detail" aria-label={`${TECH_NAME[tech]} details`} data-tech={tech}>
    <div className="research-detail-head"><h3>{TECH_NAME[tech]}</h3>
      <span className="muted">{known ? 'Researched' : option.researchable ? `Era ${eraNumeral(option.era)} · ${eraName(option.era)}` : `Locked until the ${eraName(option.era - 1)} era is finished`}</span></div>
    <div className="research-detail-columns">
      <div>
        {!known && <>
          <p><Tip content={<BreakdownTable b={option.cost} title="Research cost" unit="Science" />}><span className="research-cost-tip"><Amount n={parts.base} unit="Science" /> listed
            {diffusionLine(parts) ? ` · ${diffusionLine(parts)}` : ''} → <b><Amount n={option.cost.total} unit="Science" /></b></span></Tip></p>
          <p>{turns === null ? 'Needs Science income to finish.' : `${turns} turn${turns === 1 ? '' : 's'} at current Science${slot?.arrivalTurn ? ` · arrives turn ${slot.arrivalTurn}` : ''}`}</p>
        </>}
        {known && <p>Already researched.</p>}
      </div>
      <div className="research-impact">
        {effects.length > 0 && <p><Amounts text={effects.join('; ')} />. Applies on turns after research.</p>}
        {revealedByTech(tech).map((resource) => <p key={resource} className="research-reveal"><img src={resourceArtUrl(resource, 64) ?? ''} width={24} height={24} alt="" /> Reveals {resource} on the map.</p>)}
        {(option.income_change ?? []).map((part) => <p key={part.label}>{part.label}: <YieldAmounts yields={part.yields} /> per turn, after faction penalties.</p>)}
        {(option.effects?.upgrades ?? []).map((u) => <p key={u.unit}>Unit {u.unit}, {u.from} → {u.to}: <Tip content={<BreakdownTable b={u.after} title="Upgrade after research" unit="Gold" />}><span><Amount n={`${u.before} → ${u.after.total}`} unit="Gold" /></span></Tip>{u.eligible ? (gold >= u.after.total ? ' · affordable at current treasury' : ' · needs more Gold') : ' · first needs the upgrade technology or your own territory'}.</p>)}
        {unlock && <UnlockDetail view={view} unlock={unlock} turns={known ? null : turns} />}
      </div>
    </div>
  </section>;
}
