import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { useCelebration } from '../celebrate/MomentLane';
import { useCombatFx } from '../feel';
import { tileOnScreen } from '../map/screenPos';
import { useGame } from '../store';
import { loadWasmModule, loadedWasmModule } from '../wasmClient';
import { FactionMark } from './bits';
import { Kbd } from './index';
import { useGuide } from './guideState';
import { advisorSuggestion, useAdvisor, type AdvisorTarget } from './advisorModel';
import { useCoarsePointer } from './useCoarsePointer';
import './Advisor.css';

/** Look up mounted controls afresh: a panel can open or move without changing the suggestion. */
export function targetElement(target: Exclude<NonNullable<AdvisorTarget>, { kind: 'tile' }>): HTMLElement | null {
  const first = (...selectors: string[]) => selectors.map((s) => document.querySelector<HTMLElement>(s)).find((e) => e && e.getClientRects().length && !e.closest('[hidden]')) ?? null;
  const action = (label: RegExp) => [...document.querySelectorAll<HTMLElement>('.unit-panel button, .phone-primary-actions button')]
    .find((e) => label.test(e.getAttribute('aria-label') ?? e.textContent?.trim() ?? '') && e.getClientRects().length) ?? null;
  switch (target.name) {
    case 'research': return first('.topbar .research-chip');
    case 'turn': return target.value === 'end'
      ? first('.seatbar .cta', '.phone-turn-dock button:last-of-type')
      : first('.seatbar .cta', '.phone-turn-dock button[aria-label^="Next decision"]');
    case 'race': return first('.game .panel.vp', '.phone-vp-strip');
    case 'found': return action(/^Found City/i);
    case 'fortify': return action(/^(Fortify|Wait)/i);
    case 'skip': return action(/^Skip/i);
    case 'explore': return action(/^Auto-explore/i);
    case 'ranged': return action(/^Ranged attack/i);
    case 'build': return [...document.querySelectorAll<HTMLElement>('.city-panel .production-choice')]
      .find((e) => e.dataset.item === target.value)?.querySelector<HTMLElement>('button.prod') ?? null;
    case 'closeResearch': return first('dialog.tech-modal[open] button[aria-label="Close research"]');
    case 'tech': return [...document.querySelectorAll<HTMLElement>('dialog.tech-modal .era-card')]
      .find((e) => e.dataset.tech === target.value)?.querySelector<HTMLElement>('button[data-testid="research-now"]') ?? null;
  }
}
function targetRect(target: AdvisorTarget): { x: number; y: number; width: number; height: number; tile: boolean } | null {
  if (!target) return null;
  if (target.kind === 'tile') {
    const at = tileOnScreen(target.hex);
    const map = document.querySelector<HTMLElement>('[data-game-map]')?.getBoundingClientRect();
    if (!at || !map || at.x < map.left || at.x > map.right || at.y < map.top || at.y > map.bottom) return null;
    if (!document.elementFromPoint(at.x, at.y)?.closest('[data-game-map]')) return null;
    const r = at.radius * 1.1;
    return { x: at.x - r, y: at.y - r, width: 2 * r, height: 2 * r, tile: true };
  }
  const el = targetElement(target), r = el?.getBoundingClientRect();
  if (!r || !r.width || !r.height || r.bottom < 0 || r.top > innerHeight || r.right < 0 || r.left > innerWidth) return null;
  const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
  if (!hit || !(el === hit || el?.contains(hit))) return null;
  const pad = 6;
  return { x: r.x - pad, y: r.y - pad, width: r.width + pad * 2, height: r.height + pad * 2, tile: false };
}

function AdvisorRing({ target, still }: { target: AdvisorTarget; still: boolean }) {
  const [rect, setRect] = useState<ReturnType<typeof targetRect>>(null);
  useEffect(() => {
    if (!target) { setRect(null); return; }
    let frame = 0, last = '';
    const measure = () => {
      const next = targetRect(target);
      const key = next ? `${next.x},${next.y},${next.width},${next.height}` : '';
      if (key !== last) { last = key; setRect(next); }
      // The map camera can glide and pan; follow it on every frame.
      frame = requestAnimationFrame(measure);
    };
    measure();
    // A build card or technology further down its list is brought into view once, inside its own panel.
    const scroll = window.setTimeout(() => {
      if (target.kind !== 'control') return;
      const el = targetElement(target), r = el?.getBoundingClientRect();
      if (el && r && (r.top < 0 || r.bottom > innerHeight || !targetRect(target))) el.scrollIntoView({ block: 'center', inline: 'nearest' });
    }, 150);
    return () => { cancelAnimationFrame(frame); window.clearTimeout(scroll); };
  }, [target?.kind, target?.kind === 'tile' ? `${target.hex.q},${target.hex.r}` : target?.kind === 'control' ? `${target.name}:${target.value}` : '']);
  if (!rect) return null;
  return <div className={`advisor-ring${rect.tile ? ' map-target' : ''}${still ? ' still' : ''}`} data-testid="advisor-ring" aria-hidden="true"
    style={{ left: rect.x, top: rect.y, width: rect.width, height: rect.height }}>
      {rect.tile && <svg viewBox="0 0 100 100" width="100%" height="100%" preserveAspectRatio="none"><polygon points="50,3 97,26 97,74 50,97 3,74 3,26" fill="none" stroke="currentColor" strokeWidth="5" /></svg>}
    </div>;
}

export function Advisor() {
  const card = useRef<HTMLElement>(null);
  const touch = useCoarsePointer();
  const view = useGame((s) => s.views?.[s.seat]);
  const seat = useGame((s) => s.seat);
  const orders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
  const humans = useGame((s) => s.humans);
  const selection = useGame((s) => s.selection);
  const mode = useGame((s) => s.mode);
  const techOpen = useGame((s) => s.techOpen);
  const client = useGame((s) => s.client);
  const animationsOn = useCombatFx((s) => s.enabled);
  const paused = useGame((s) => !!s.busy || !!s.endTurnConfirmation || s.reportOpen || s.diplomacyOpen || !!s.ready[s.seat] || !!s.place ||
    (s.client?.kind === 'remote' && s.net?.conn !== 'open'));
  const [blockingOverlay, setBlockingOverlay] = useState(false);
  useEffect(() => {
    const check = () => setBlockingOverlay(!!document.querySelector('[data-game-overlay]:not([data-dismissible])'));
    check();
    const observer = new MutationObserver(check);
    observer.observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ['data-game-overlay', 'data-dismissible'] });
    const unsubscribe = useGame.subscribe(check);
    return () => { observer.disconnect(); unsubscribe(); };
  }, []);
  const guideOpen = useGuide((s) => s.open);
  const { active: celebration, holding } = useCelebration();
  const { enabled, seen, skipped, skippedTurn, markSeen, skip, setEnabled } = useAdvisor();
  const [, refreshPlan] = useState(0);
  useEffect(() => {
    if (!view || !enabled || loadedWasmModule()) return;
    let active = true;
    void loadWasmModule().then(() => { if (active) refreshPlan((n) => n + 1); }).catch(() => { /* Basic guidance remains available. */ });
    return () => { active = false; };
  }, [view, enabled]);
  const hidden = paused || blockingOverlay || guideOpen || holding || !!celebration || !enabled || !view || view.finished;
  const turnLabel = document.querySelector<HTMLElement>('.seatbar .cta-label')?.textContent?.trim()
    ?? document.querySelector<HTMLElement>('.phone-turn-dock button[aria-label^="Next decision"]')?.textContent?.trim() ?? undefined;
  const suggestion = view && enabled ? advisorSuggestion({ view, orders: orders[seat] ?? [], errors: errors[seat], selection,
    humans: humans.length, remote: client?.kind === 'remote', touch, techOpen, mode, turnLabel, local: useGame.getState(),
    forecast: client ? (unit, at, mode) => client.forecast(view.me, unit, at, mode) : undefined }, seen,
    skippedTurn === view.turn ? skipped : []) : null;
  const [modal, setModal] = useState<HTMLDialogElement | null>(null);
  useEffect(() => { setModal(techOpen ? document.querySelector<HTMLDialogElement>('dialog.tech-modal[open]') : null); }, [techOpen]);
  // The card's top edge in pixels when the usual slot would cover the step's target or the selected tile; null keeps the CSS slot.
  const [cardTop, setCardTop] = useState<number | null>(null);
  const [dialogRect, setDialogRect] = useState<DOMRect | null>(null);
  useEffect(() => setCardTop(null), [suggestion?.id, suggestion?.current]);
  useLayoutEffect(() => {
    if (hidden || !suggestion) return;
    let frame = 0, previous = '';
    const measure = () => {
      const el = card.current;
      if (modal) {
        const r = modal.getBoundingClientRect(), key = `${r.x},${r.y},${r.width},${r.height}`;
        if (key !== previous) { previous = key; setDialogRect(r); }
      } else if (el) {
        const box = el.getBoundingClientRect();
        const step = suggestion.steps[suggestion.current];
        let target: DOMRect | null = null;
        if (step?.target?.kind === 'control') target = targetElement(step.target)?.getBoundingClientRect() ?? null;
        else if (step?.target?.kind === 'tile') {
          const p = tileOnScreen(step.target.hex);
          if (p) target = new DOMRect(p.x - p.radius, p.y - p.radius, 2 * p.radius, 2 * p.radius);
        }
        let selectedTile: DOMRect | null = null;
        if (selection) {
          const entity = selection.kind === 'unit' ? view?.units.find((u) => u.id === selection.id) : selection.kind === 'city' ? view?.cities.find((c) => c.id === selection.id) : null;
          const p = entity && tileOnScreen(entity.pos);
          if (p) selectedTile = new DOMRect(p.x - p.radius, p.y - p.radius, 2 * p.radius, 2 * p.radius);
        }
        // Also keep clear of the map tools and the player's own cities with their banners (drawn above the tile).
        const tools = document.querySelector('.map-lenses')?.getBoundingClientRect() ?? null;
        const ownCities = (view?.cities ?? []).filter((c) => c.owner === view?.me).map((c) => tileOnScreen(c.pos))
          .filter((p): p is NonNullable<typeof p> => !!p).map((p) => new DOMRect(p.x - p.radius, p.y - 2 * p.radius, 2 * p.radius, 3 * p.radius));
        const blockers = [target, selectedTile, tools, ...ownCities].filter((r): r is DOMRect => !!r && r.width > 0 && r.right > box.left && r.left < box.right);
        const clearOf = (rects: DOMRect[]) => (top: number) => rects.every((r) => top + box.height + 8 <= r.top || top >= r.bottom + 8);
        // The step's target and the selection must stay visible; the rest is avoided when there is room.
        const must = blockers.filter((r) => r === target || r === selectedTile);
        const home = 76;
        const floor = innerHeight - box.height - 8;
        // Prefer the usual slot, then just below a blocker, then just above one.
        const options = [home, ...blockers.map((r) => r.bottom + 12), ...blockers.map((r) => r.top - 12 - box.height)]
          .filter((top) => top >= 64 && top <= floor);
        const cities = blockers.filter((r) => r !== tools);
        const best = options.find(clearOf(blockers)) ?? options.find(clearOf(cities)) ?? options.find(clearOf(must));
        const next = best === undefined || best === home ? null : Math.round(best);
        if (next !== cardTop) setCardTop(next);
      }
      frame = requestAnimationFrame(measure);
    };
    measure();
    return () => cancelAnimationFrame(frame);
  }, [hidden, suggestion?.id, suggestion?.current, modal, selection, view, cardTop]);
  useLayoutEffect(() => {
    const el = card.current, game = document.querySelector<HTMLElement>('.game');
    if (!el || !game) return;
    const measure = () => game.style.setProperty('--hint-bottom', `${el.getBoundingClientRect().bottom - game.getBoundingClientRect().top}px`);
    measure();
    const resize = new ResizeObserver(measure);
    resize.observe(el);
    return () => { resize.disconnect(); game.style.removeProperty('--hint-bottom'); };
  }, [hidden, suggestion?.id, suggestion?.current, modal]);
  if (hidden || !suggestion) return null;
  const step = suggestion.steps[suggestion.current];
  const welcome = suggestion.id === 'welcome';
  const lesson = suggestion.acknowledge && !welcome;
  // The player's own seal names the speaker; no eyebrow word.
  const seal = view && <span className="advisor-seal" aria-hidden="true"><FactionMark faction={view.players.find((p) => p.id === view.me)?.faction ?? 'Switzerland'} size={20} /></span>;
  const content = <>
    {/* Inside a dialog, only ring a control in that dialog: anything behind it can't be clicked. */}
    <AdvisorRing target={modal && !(step?.target?.kind === 'control' && modal.contains(targetElement(step.target))) ? null : step?.target ?? null} still={!animationsOn} />
    <aside ref={card} className={`advisor-card${welcome ? ' welcome' : ''}${modal ? ' in-dialog' : ''}`}
      style={modal && dialogRect ? { left: dialogRect.left, bottom: innerHeight - dialogRect.bottom, width: dialogRect.width } : cardTop !== null ? { top: cardTop, bottom: 'auto' } : undefined}
      data-overlay-slot="story-hint" data-testid="advisor" data-advisor-id={suggestion.id} aria-label="Advisor" onKeyDown={(e) => e.stopPropagation()}>
      {modal && seal}
      <div className="advisor-copy" role="status" aria-live="polite" aria-atomic="true">
        <h3>{seal}<span>{suggestion.title}</span></h3>
        <p>{suggestion.why}</p>
      </div>
      {step && <div className="advisor-step">{suggestion.steps.length > 1 && <span className="advisor-count">Step {suggestion.current + 1} of {suggestion.steps.length}</span>}
        <strong>{step.instruction}</strong>{step.key && <Kbd>{step.key}</Kbd>}{modal && <span className="advisor-dialog-why">{suggestion.why}</span>}
      </div>}
      <div className="advisor-actions">
        {welcome ? <button className="btn primary" onClick={() => markSeen('welcome')}>Continue {!touch && <Kbd>Return</Kbd>}</button>
          : lesson ? <button className="btn primary" onClick={() => markSeen(suggestion.id)}>Got it</button>
            : <button className="advisor-link" onClick={() => skip(suggestion.id, view!.turn)}>Not now</button>}
        <button className="advisor-link" onClick={() => setEnabled(false)}>Turn off</button>
      </div>
    </aside>
  </>;
  return modal ? createPortal(content, modal) : content;
}

export function AdvisorSettings() {
  const { enabled, setEnabled } = useAdvisor();
  return <div className="hint-settings" onKeyDown={(e) => { if (e.key !== 'Escape') e.stopPropagation(); }}>
    <label><input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} />Show the advisor</label>
    <div className="hint-settings-foot"><span>On by default. Turn it off any time.</span></div>
  </div>;
}
