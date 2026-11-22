import { Amounts } from '../ui/Amounts';
import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import type { Hex, PlayerPublic, Resource, SupplyView } from '../types';
import { RESOURCE_INFO, eraResource, lackedResources, resourceUsersThisEra, revealHint, STRATEGIC_RESOURCES, supplyTotalsText } from '../resources';
import { playerColor, rgba } from '../theme';
import type { SettlementScore } from '../types';
import { LENSES, LENS_ALPHA, LENS_CAVEATS, LENS_LEGENDS, LENS_NAMES, type MapLens } from './lenses';
import { OVERLAYS, OVERLAY_EMPTY, OVERLAY_HINTS, OVERLAY_KEYS, OVERLAY_LEGENDS, OVERLAY_NAMES, toggleOverlay, useOverlays, type Overlay } from './overlays';
import { Tip } from '../ui/Tip';
import { factionName } from '../data';
import { SETTLE_BANDS, bandRange } from './settleAdvice';
import { Icon, YieldIcon } from '../ui/bits';
import { usePhoneLayout } from '../ui/usePhoneLayout';
import { useCoarsePointer } from '../ui/useCoarsePointer';
import './lenses.css';

// A lens or overlay with nothing to mark must say so, or it looks like it did not switch (pillar 9).
const EMPTY: Partial<Record<MapLens, string>> = {
  threat: 'No known enemy military can reach any tile next turn.',
  discovery: 'No unexplored edges or known ruins in view.',
};

/** A short window (the user's 1000x600 laptop) has no room for overlay keys beside a unit panel, so they open on request. */
function useShortWindow() {
  const query = '(max-height: 700px)';
  const [short, setShort] = useState(() => window.matchMedia(query).matches);
  useEffect(() => {
    const media = window.matchMedia(query);
    const update = () => setShort(media.matches);
    update();
    media.addEventListener?.('change', update);
    return () => media.removeEventListener?.('change', update);
  }, []);
  return short;
}

/** `collapsed`: a unit or city is selected, so the dock yields height to its panel and
 * shows only the "Lens: X" toggle (as on phones) until the selection is cleared. */
export function LensSwitcher({ lens, onChange, sites, onInspect, settlerId, players, me, visibleBarbarians = false, supply = [], era, empty = false, overlayEmpty = {}, collapsed = false }: {
  lens: MapLens; onChange(lens: MapLens): void; sites: SettlementScore[]; onInspect(hex: Hex): void; settlerId: number | null; players: PlayerPublic[]; me?: number; visibleBarbarians?: boolean; supply?: SupplyView[]; era?: number; empty?: boolean; overlayEmpty?: Partial<Record<Overlay, boolean>>; collapsed?: boolean;
}) {
  const overlays = useOverlays();
  const overlaysOn = OVERLAYS.filter((o) => overlays[o]);
  const short = useShortWindow();
  const [keyOpen, setKeyOpen] = useState(false);
  const hasKeys = overlays.resources || overlays.borders;
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
  const persistentLegend: 'settle' | 'resources' | null = !phone ? null : lens === 'settle' ? 'settle' : overlays.resources ? 'resources' : null;
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
  const settleLegend = <div className="settle-legend">
    <p>Better places to found a city are darker green. {touch ? 'Tap' : 'Hover'} a tile for why. No tint: not a site.</p>
    <div className="settle-scale" aria-label="Site quality, from poor to exceptional">{SETTLE_BANDS.map((band) => <span key={band} data-settle-band={band} title={`${band}: site quality ${bandRange(band)}. The same scale on every map.`}><i className={`settle-swatch band-${band.toLowerCase()}`} aria-hidden="true" /><em>{band}</em></span>)}</div>
  </div>;
  return <section ref={panel} className="map-lenses" data-overlay-slot="map-tools" data-compact={compact} data-chip={chip || undefined} data-open={compact && open} aria-label="Map options" onKeyDown={(e) => {
    if (compact && open && e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); }
  }}>
    {compact && <button ref={toggle} type="button" className="lens-toggle" aria-label={chip ? `Lens: ${LENS_NAMES[lens]}` : undefined} aria-expanded={open} aria-controls={contentId} onClick={() => setOpen(!open)}>{chip ? 'Lens' : <>Lens: {LENS_NAMES[lens]}</>}{overlaysOn.length > 0 && <span className="lens-toggle-overlays"> · {overlaysOn.map((o) => OVERLAY_NAMES[o]).join(', ')}</span>}</button>}
    <div id={contentId} className="lens-content" data-overlay-slot={phone && open ? 'map-inspector' : undefined} hidden={compact && !open} role={compact ? 'region' : undefined} aria-label={compact ? 'Choose map options' : undefined}>
    {compact && <div className="lens-sheet-head"><b>Choose map options</b><button className="btn quiet" aria-label="Close map options" onClick={close}>Close</button></div>}
    <div className="lens-group-label" id={`${contentId}-overlays`}>Overlays {overlays.yields
      ? <Tip placement="right" content={OVERLAY_HINTS.yields}><small className="lens-yield-line"><span className="lens-yield-key">{(['food', 'production', 'gold', 'science'] as const).filter((k) => k !== 'production' || players.find((p) => p.id === me)?.faction !== 'Egypt').map((k) => <YieldIcon key={k} k={k} size={13} />)}</span> {OVERLAY_LEGENDS.yields}</small></Tip>
      : <small>stay on over any lens</small>}</div>
    <div role="group" aria-labelledby={`${contentId}-overlays`} className="overlay-toggles">
      {OVERLAYS.map((o) => <Tip key={o} className="overlay-tip" placement="top" content={OVERLAY_HINTS[o]}>
        <button type="button" role="checkbox" aria-checked={overlays[o]} aria-keyshortcuts={OVERLAY_KEYS[o]} data-overlay={o} className="overlay-toggle" onClick={() => toggleOverlay(o)}>
          <i className="overlay-box" aria-hidden="true">{overlays[o] && <svg viewBox="0 0 10 10"><path d="M1.8 5.4 4.1 7.6 8.3 2.6" /></svg>}</i>
          {OVERLAY_NAMES[o]}<kbd>{OVERLAY_KEYS[o]}</kbd>
        </button>
      </Tip>)}
    </div>
    <div className="lens-group-label" id={`${contentId}-lens`}>Lens <small>one at a time</small>
      {short && hasKeys && <button type="button" className="lens-key-toggle" aria-expanded={keyOpen} aria-controls={`${contentId}-key`} onClick={() => setKeyOpen(!keyOpen)}>Overlay key</button>}
    </div>
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
    {hasKeys && (!short || keyOpen) && <div id={`${contentId}-key`} className="lens-key-content" aria-live="polite">
      {overlays.resources && <div className="lens-legend overlay-legend">
        {overlayEmpty.resources && <p className="lens-empty"><b>{OVERLAY_EMPTY.resources}</b></p>}
        <ResourceKey supply={supply} era={era} faction={players.find((p) => p.id === me)?.faction} swatches={persistentLegend !== 'resources'} />
      </div>}
      {overlays.borders && <div className="lens-legend lens-border-key">
        <p><Tip placement="right" content={OVERLAY_HINTS.borders}><span className="lens-legend-line">{OVERLAY_LEGENDS.borders}</span></Tip></p>
        {overlayEmpty.borders && <p className="lens-empty"><b>{OVERLAY_EMPTY.borders}</b></p>}
        <div className="lens-owner-key" aria-label="Empire numbers">{players.map((p) => <span key={p.id} title={p.id === me ? 'You' : factionName(p.faction)} aria-label={`${p.id + 1}: ${p.id === me ? 'You' : factionName(p.faction)}`}><i style={{ background: rgba(playerColor(p.id), LENS_ALPHA), borderColor: playerColor(p.id) }} />{p.id + 1}</span>)}</div>
      </div>}
    </div>}
    {lens !== 'default' && <div className="lens-legend" aria-live="polite">
      {lens !== 'settle' && <p>{LENS_CAVEATS[lens] ? <Tip placement="right" content={LENS_CAVEATS[lens]}><span className="lens-legend-line">{LENS_LEGENDS[lens]}</span></Tip> : LENS_LEGENDS[lens]}</p>}
      {empty && EMPTY[lens] && <p className="lens-empty"><b>{lens === 'threat' && players.find((p) => p.id === me)?.faction === 'Celts' && visibleBarbarians ? "Nothing in sight can attack you next turn. Barbarians don't attack you." : EMPTY[lens]}</b></p>}
      {lens === 'settle' && <>
        {!persistentLegend && settleLegend}
        {best.length ? <><p className="settle-top-key" data-testid="settle-top-key">{settlerId === null ? 'Top three from your capital, marked on the map:' : 'Top three for this Settler, marked on the map:'}</p><div className="settle-recommendations" aria-label="Recommended settlement sites">{best.map((s) => <button className="btn" key={s.rank} data-settle-rank={s.rank} onClick={() => { if (compact) close(); onInspect(s.pos); }} aria-label={s.rank === 1 ? 'Inspect best spot' : `Inspect recommended site ${s.rank}`}>{s.rank === 1 ? 'Best spot' : `#${s.rank}`}</button>)}</div></>
          : <p>No safe reachable recommendations. Explore more land or clear nearby threats.</p>}
        {settlerId === null && <p>Travel is counted from your capital. Select or train a Settler to compare from its position.</p>}
      </>}
    </div>}
    </div>
    {persistentLegend && <div ref={legend} className="settle-persistent-legend" data-overlay-slot="map-tools-legend">{persistentLegend === 'settle' ? settleLegend : <ResourceSwatches supply={supply} tip={false} />}</div>}
  </section>;
}

/** The Resources overlay key, one row: the ring shape tells whose land a resource is in; the ring's colour is its kind
 * (hover for that, for the dashed "not improved yet" ring, and for what reveals a hidden strategic resource). */
function ResourceSwatches({ supply, tip = true }: { supply: SupplyView[]; tip?: boolean }) {
  const hidden = STRATEGIC_RESOURCES.filter((r) => supply.some((s) => s.resource === r && !s.revealed));
  const wanted = lackedResources(supply).size > 0;
  const row = <div className="lens-key-row"><b>Rings</b>
    <span><i className="ring-yours" />Yours</span>
    <span><i className="ring-free" />Unclaimed</span>
    <span><i className="ring-foreign" />Foreign</span>
    {wanted && <span><i className="ring-wanted" aria-hidden>!</i>You lack it</span>}
  </div>;
  return <div className="lens-resource-key">
    {tip ? <Tip placement="right" content={<>
      <div>Ring shape: whose land the resource is in. Ring colour: green is a bonus resource, red a strategic one. A dashed dark ring means yours but not improved yet.</div>
      <div>Strategic resources begin with Iron in the Classical era. Deposits inside your borders supply the buildings and units that need them. Copper is a bonus tile resource.</div>
      {wanted && <div>A red "!" marks a deposit of a strategic resource you lack that another empire holds or nobody does.</div>}
      {hidden.map((r) => <div key={r}>{revealHint(r)}</div>)}
    </>}>{row}</Tip> : row}
  </div>;
}

/** Below the key: your revealed strategic resources, with deposits and supply in use. On phones the
 * key is the compact persistent row (no tip), so the sheet also says what reveals the hidden ones. */
function ResourceKey({ supply, era, faction, swatches = true }: { supply: SupplyView[]; era?: number; faction?: string; swatches?: boolean }) {
  const revealed = supply.filter((s) => s.revealed || !swatches);
  // Your era's resource first, then the ones you lack, so a short window never cuts off the line that matters.
  const lacked = lackedResources(supply), mine = era === undefined ? null : eraResource(era);
  const rank = (r: Resource) => (r === mine ? 0 : lacked.has(r) ? 1 : 2);
  const order = [...STRATEGIC_RESOURCES].sort((a, b) => rank(a) - rank(b));
  return <>
    {swatches && <ResourceSwatches supply={supply} />}
    {revealed.length > 0 && <ul className="lens-supply" aria-label="Your strategic resources">
      {order.map((r) => {
        const s = revealed.find((p) => p.resource === r);
        if (!s) return null;
        return <li key={r}>{s.revealed ? <><Icon name={`resource-${r.toLowerCase()}`} size={14} /> <b>{RESOURCE_INFO[r].name}</b>: {s.supplied.parts.length} {s.supplied.parts.length === 1 ? 'deposit' : 'deposits'} in your borders · <Amounts text={supplyTotalsText(s)} /> · Used for {resourceUsersThisEra(r, era ?? 0, faction)}</> : revealHint(r)}</li>;
      })}
    </ul>}
  </>;
}
