import { COLONIA_RING_KEY, COLONIA_SITE_RULE } from '../goalCities';
import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import type { Hex, PlayerPublic } from '../types';
import type { SettlementScore } from '../types';
import { LENSES, LENS_NAMES, type MapLens } from './lenses';
import { OVERLAYS, OVERLAY_HINTS, OVERLAY_KEYS, OVERLAY_LEGENDS, OVERLAY_NAMES, toggleOverlay, useOverlays } from './overlays';
import { Tip } from '../ui/Tip';
import { SETTLE_BANDS, bandRange } from './settleAdvice';
import { YieldIcon } from '../ui/bits';
import { usePhoneLayout } from '../ui/usePhoneLayout';
import { useCoarsePointer } from '../ui/useCoarsePointer';
import './lenses.css';

/** `collapsed`: a unit or city is selected, so the dock yields height to its panel and
 * shows only the "Lens: X" toggle (as on phones) until the selection is cleared. */
export function LensSwitcher({ lens, onChange, sites, onInspect, settlerId, players, me, collapsed = false, coloniaFounder = false, founderName = 'Settler' }: {
  lens: MapLens; onChange(lens: MapLens): void; sites: SettlementScore[]; onInspect(hex: Hex): void; settlerId: number | null; players: PlayerPublic[]; me?: number; collapsed?: boolean;
  /** Rome: the selected unit founds the Colonia Victrix on a site that counts; the legend names the gilt ring, or says what a site needs when none is in view. */
  coloniaFounder?: boolean;
  /** The selected founder's name ("Spearman", "Settler"), so the legend names the unit actually selected. */
  founderName?: string;
}) {
  const overlays = useOverlays();
  const overlaysOn = OVERLAYS.filter((o) => overlays[o]);
  const panel = useRef<HTMLElement>(null);
  const toggle = useRef<HTMLButtonElement>(null);
  const legend = useRef<HTMLDivElement>(null);
  const phone = usePhoneLayout();
  const touch = useCoarsePointer();
  const compact = phone || collapsed;
  // Beside a selection's panel on a computer, the Default lens with no overlay has nothing to say: a small "Lens" chip
  // beside the panel keeps the chooser one click away and gives the panel the dock's height (the 1000x600 laptop).
  const chip = collapsed && !phone && lens === 'default' && overlaysOn.length === 0;
  // Phones: the lens chooser is a small scrolling sheet, so a lens whose colours need a key keeps it on screen in one row.
  const persistentLegend = phone && lens === 'settle';
  const [open, setOpen] = useState(false);
  const contentId = useId();
  const close = () => { setOpen(false); toggle.current?.focus({ preventScroll: true }); };
  useEffect(() => {
    if (!open) return;
    const dismiss = (event: PointerEvent) => {
      if (!panel.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener('pointerdown', dismiss);
    return () => document.removeEventListener('pointerdown', dismiss);
  }, [open]);
  useEffect(() => { if (!compact) setOpen(false); }, [compact]);
  useLayoutEffect(() => {
    if (compact && open) panel.current?.querySelector<HTMLButtonElement>('[aria-checked="true"]')?.focus({ preventScroll: true });
  }, [compact, open]);
  useLayoutEffect(() => {
    const el = panel.current!, game = el.closest<HTMLElement>('.game');
    if (!game) return;
    const measure = () => game.style.setProperty('--lens-height', chip ? '0px' : `${el.getBoundingClientRect().height}px`);
    measure();
    const resize = new ResizeObserver(measure);
    resize.observe(el);
    return () => { resize.disconnect(); game.style.removeProperty('--lens-height'); };
    // Re-measure in the same commit that collapses or expands the dock: the ResizeObserver only
    // fires after the next frame, so a city panel would otherwise paint (and be measured) one
    // frame too short, against the expanded dock's height.
  }, [compact, chip]);
  useLayoutEffect(() => {
    const game = panel.current?.closest<HTMLElement>('.game'), el = legend.current;
    if (!game || !el) return;
    // Reserve a row between the phone's tools and story slot, even with the chooser closed.
    const measure = () => game.style.setProperty('--settle-legend-height', `${el.getBoundingClientRect().height + 8}px`);
    measure();
    const resize = new ResizeObserver(measure);
    resize.observe(el);
    return () => { resize.disconnect(); game.style.removeProperty('--settle-legend-height'); };
  }, [persistentLegend]);
  const best = sites.filter((s) => s.rank !== null).sort((a, b) => a.rank! - b.rank!);
  const coloniaBest = coloniaFounder && best.length > 0 && best.every((s) => s.colonia);
  const settleLegend = <div className="settle-legend">
    <p>Better places to found a city are darker green. {touch ? 'Tap' : 'Hover'} a tile for why. No tint: not a site.</p>
    <div className="settle-scale" aria-label="Site quality, from poor to exceptional">{SETTLE_BANDS.map((band) => <span key={band} data-settle-band={band} title={`${band}: site quality ${bandRange(band)}. The same scale on every map.`}><i className={`settle-swatch band-${band.toLowerCase()}`} aria-hidden="true" /><em>{band}</em></span>)}</div>
    {coloniaFounder && (sites.some((s) => s.colonia)
      ? <p className="settle-colonia-key" data-testid="settle-colonia-key"><i className="settle-ring-gilt" aria-hidden="true" />{COLONIA_RING_KEY}</p>
      : <p className="settle-colonia-key none" data-testid="settle-colonia-none"><i className="settle-ring-empty" aria-hidden="true" />No site you know founds your Colonia Victrix. {COLONIA_SITE_RULE}</p>)}
  </div>;
  return <section ref={panel} className="map-lenses" data-overlay-slot="map-tools" data-compact={compact} data-chip={chip || undefined} data-open={compact && open} aria-label="Map options" onKeyDown={(e) => {
    if (compact && open && e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); }
  }}>
    {compact && <button ref={toggle} type="button" className="lens-toggle" aria-label={chip ? `Lens: ${LENS_NAMES[lens]}` : undefined} aria-expanded={open} aria-controls={contentId} onClick={() => setOpen(!open)}>{chip ? 'Lens' : <>Lens: {LENS_NAMES[lens]}</>}{overlaysOn.length > 0 && <span className="lens-toggle-overlays"> · {overlaysOn.map((o) => OVERLAY_NAMES[o]).join(', ')}</span>}</button>}
    <div id={contentId} className="lens-content" data-overlay-slot={phone && open ? 'map-inspector' : undefined} hidden={compact && !open} role={compact ? 'region' : undefined} aria-label={compact ? 'Choose map options' : undefined}>
    {compact && <div className="lens-sheet-head"><b>Choose map options</b><button className="btn quiet" aria-label="Close map options" onClick={close}>Close</button></div>}
    <div className="lens-bar">
    <div role="radiogroup" aria-label="Map lens" className="lens-options" onKeyDown={(e) => {
      const step = ['ArrowRight', 'ArrowDown'].includes(e.key) ? 1 : ['ArrowLeft', 'ArrowUp'].includes(e.key) ? -1 : 0;
      if (!step && e.key !== 'Home' && e.key !== 'End') return;
      e.preventDefault();
      const index = e.key === 'Home' ? 0 : e.key === 'End' ? LENSES.length - 1 : (LENSES.indexOf(lens) + step + LENSES.length) % LENSES.length;
      onChange(LENSES[index]);
      e.currentTarget.querySelectorAll<HTMLButtonElement>('[role="radio"]')[index].focus();
    }}>
      {LENSES.map((id) => <button key={id} type="button" role="radio" aria-checked={id === lens} tabIndex={id === lens ? 0 : -1} onClick={() => { onChange(id); if (compact) close(); }}>{LENS_NAMES[id]}</button>)}
    </div>
    <Tip className="overlay-tip" placement="top" content={OVERLAY_HINTS.yields}>
      <button type="button" role="checkbox" aria-checked={overlays.yields} aria-keyshortcuts={OVERLAY_KEYS.yields} data-overlay="yields" className="overlay-toggle" onClick={() => toggleOverlay('yields')}>
        <i className="overlay-box" aria-hidden="true">{overlays.yields && <svg viewBox="0 0 10 10"><path d="M1.8 5.4 4.1 7.6 8.3 2.6" /></svg>}</i>
        {OVERLAY_NAMES.yields}<kbd>{OVERLAY_KEYS.yields}</kbd>
      </button>
    </Tip>
    </div>
    {overlays.yields && <Tip placement="right" content={OVERLAY_HINTS.yields}><small className="lens-yield-line"><span className="lens-yield-key">{(['food', 'production', 'gold', 'science'] as const).filter((k) => k !== 'production' || players.find((p) => p.id === me)?.faction !== 'Egypt').map((k) => <YieldIcon key={k} k={k} size={13} />)}</span> {OVERLAY_LEGENDS.yields}</small></Tip>}
    {lens === 'settle' && <div className="lens-legend" aria-live="polite">
      <>
        {!persistentLegend && settleLegend}
        {best.length ? <><p className="settle-top-key" data-testid="settle-top-key">{settlerId === null ? 'Top three from your capital, marked on the map:' : coloniaBest ? 'Top three Colonia sites for this veteran, marked on the map:' : `Top three for this ${founderName}, marked on the map:`}</p><div className={`settle-recommendations${coloniaBest ? ' colonia' : ''}`} aria-label="Recommended settlement sites">{best.map((s) => <button className="btn" key={s.rank} data-settle-rank={s.rank} onClick={() => { if (compact) close(); onInspect(s.pos); }} aria-label={s.rank === 1 ? (coloniaBest ? 'Inspect best Colonia site' : 'Inspect best spot') : `Inspect recommended site ${s.rank}`}>{s.rank === 1 ? (coloniaBest ? 'Best Colonia site' : 'Best spot') : `#${s.rank}`}</button>)}</div></>
          : <p>No safe reachable recommendations. Explore more land or clear nearby threats.</p>}
        {settlerId === null && <p>Travel is counted from your capital. Select or train a Settler to compare from its position.</p>}
      </>
    </div>}
    </div>
    {persistentLegend && <div ref={legend} className="settle-persistent-legend" data-overlay-slot="map-tools-legend">{settleLegend}</div>}
  </section>;
}
