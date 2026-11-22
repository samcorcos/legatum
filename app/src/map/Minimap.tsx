import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { useGame } from '../store';
import type { PlayerView } from '../types';
import { playerColor } from '../theme';
import { usePhoneLayout } from '../ui/usePhoneLayout';
import { centerOnWorld, mapViewport, onMapCamera } from './camera';
import { MAP_STYLES } from './style';
import './minimap.css';
import { useCityAlarms } from '../ui/useCityAlarms';
import { cityNear, drawMinimapBase, drawMinimapOverlay, minimapLayout, miniToWorld, type MinimapInk } from './minimapView';

// Civ-style minimap: the whole map at small scale, from the viewer's knowledge only. Desktop and
// tablet get a corner panel under the turn dock (or beside it where the bottom row is free);
// phones get a Map button that opens a larger overview. See map/minimap.ts for what is drawn.

const MAP_ICON = <svg viewBox="0 0 20 20" aria-hidden="true"><path d="M2 4.5l5-2 6 2 5-2v13l-5 2-6-2-5 2z M7 2.5v13 M13 4.5v13" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinejoin="round" /></svg>;

const COLLAPSED_KEY = 'legatum.minimap.collapsed';

function readCollapsed(): boolean {
  try { return globalThis.localStorage?.getItem(COLLAPSED_KEY) === '1'; } catch { return false; }
}
function writeCollapsed(v: boolean) {
  try { globalThis.localStorage?.setItem(COLLAPSED_KEY, v ? '1' : '0'); } catch { /* storage blocked */ }
}

const INK: MinimapInk = { dark: '#2b2419', light: '#fffaf0' };

/** The map hands focus back to itself, so keyboard shortcuts keep working after a minimap click. */
function returnFocusToMap() {
  document.querySelector<HTMLElement>('[data-game-map]')?.focus({ preventScroll: true });
}

interface CanvasProps {
  view: PlayerView;
  maxW: number;
  maxH: number;
  /** Called with a world point: every step of a drag, then once more with `done` on release. */
  onPick(x: number, y: number, done: boolean): void;
  label: string;
}

function MinimapCanvas({ view, maxW, maxH, onPick, label }: CanvasProps) {
  const ref = useRef<HTMLCanvasElement>(null);
  const [dpr, setDpr] = useState(() => Math.min(2, globalThis.devicePixelRatio || 1));
  const [tip, setTip] = useState<{ name: string; x: number; y: number } | null>(null);
  const drag = useRef<number | null>(null);
  const layout = useMemo(() => minimapLayout(view, maxW, maxH), [view.width, view.height, maxW, maxH]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const media = matchMedia(`(resolution: ${dpr}dppx)`);
    const update = () => setDpr(Math.min(2, globalThis.devicePixelRatio || 1));
    media.addEventListener?.('change', update);
    return () => media.removeEventListener?.('change', update);
  }, [dpr]);

  // Terrain, fog and territory change only with the view (a new turn): cache them.
  const base = useMemo(() => {
    const c = document.createElement('canvas');
    c.width = Math.round(layout.w * dpr);
    c.height = Math.round(layout.h * dpr);
    const ctx = c.getContext('2d');
    if (ctx) drawMinimapBase(ctx, view, layout, MAP_STYLES.atlas, dpr, playerColor);
    return c;
  }, [view.tiles, view.me, layout, dpr]); // eslint-disable-line react-hooks/exhaustive-deps

  // Dots and the viewport frame: redrawn when the view changes or the main camera moves.
  const cityAlarms = useCityAlarms();
  const alarms = useMemo(() => new Map(cityAlarms.map((a) => [a.cityId, a.kind] as const)), [cityAlarms]);
  const latest = useRef({ view, base, layout, dpr, alarms });
  latest.current = { view, base, layout, dpr, alarms };
  useEffect(() => {
    let frame = 0;
    const paint = () => {
      frame = 0;
      const canvas = ref.current, ctx = canvas?.getContext('2d');
      if (!canvas || !ctx) return;
      const f = latest.current;
      drawMinimapOverlay(ctx, f.base, f.view, f.layout, f.dpr, playerColor, INK, mapViewport(), f.alarms);
    };
    const schedule = () => { if (!frame) frame = requestAnimationFrame(paint); };
    paint();
    const off = onMapCamera(schedule);
    return () => { off(); cancelAnimationFrame(frame); };
  }, [view, base, layout, dpr, alarms]);

  const local = (e: { clientX: number; clientY: number }) => {
    const r = ref.current!.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };
  const pick = (e: React.PointerEvent, done: boolean) => {
    const p = local(e);
    const w = miniToWorld(layout, Math.max(0, Math.min(layout.w, p.x)), Math.max(0, Math.min(layout.h, p.y)));
    onPick(w.x, w.y, done);
  };

  return (
    <div className="minimap-canvas-wrap" style={{ width: layout.w, height: layout.h }}>
      <canvas
        ref={ref}
        className="minimap-canvas"
        data-testid="minimap-canvas"
        role="img"
        aria-label={label}
        width={Math.round(layout.w * dpr)}
        height={Math.round(layout.h * dpr)}
        style={{ width: layout.w, height: layout.h }}
        onPointerDown={(e) => {
          if (e.button !== 0) return;
          e.preventDefault();
          drag.current = e.pointerId;
          e.currentTarget.setPointerCapture?.(e.pointerId);
          e.currentTarget.dataset.dragging = 'true';
          setTip(null);
          pick(e, false);
        }}
        onPointerMove={(e) => {
          if (drag.current === e.pointerId) { pick(e, false); return; }
          if (e.pointerType !== 'mouse') return;
          const p = local(e);
          const city = cityNear(view, layout, p.x, p.y);
          setTip(city ? { name: city.name, x: p.x, y: p.y } : null);
        }}
        onPointerUp={(e) => {
          if (drag.current !== e.pointerId) return;
          drag.current = null;
          delete e.currentTarget.dataset.dragging;
          if (e.currentTarget.hasPointerCapture?.(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
          pick(e, true);
        }}
        onPointerCancel={(e) => { drag.current = null; delete e.currentTarget.dataset.dragging; }}
        onPointerLeave={() => setTip(null)}
        onContextMenu={(e) => e.preventDefault()}
      />
      {tip && <span className="minimap-tip" aria-hidden="true" style={{ left: tip.x, top: tip.y }}>{tip.name}</span>}
    </div>
  );
}

function Legend() {
  return (
    <ul className="minimap-legend" aria-label="Minimap legend">
      <li><i className="mk city" aria-hidden="true" />City</li>
      <li><i className="mk capital" aria-hidden="true" />Capital</li>
      <li><i className="mk scout" aria-hidden="true" />Your scout</li>
      <li><i className="mk unit" aria-hidden="true" />Unit</li>
      <li><i className="mk view" aria-hidden="true" />Your view</li>
    </ul>
  );
}

/** Read the panel's size budget from CSS (it differs between the stacked and bottom-row slots). */
function useBudget(el: React.RefObject<HTMLElement | null>, fallback: { w: number; h: number }) {
  const [budget, setBudget] = useState(fallback);
  useLayoutEffect(() => {
    const read = () => {
      const node = el.current;
      if (!node) return;
      const cs = getComputedStyle(node);
      const w = parseFloat(cs.getPropertyValue('--minimap-max-w')) || fallback.w;
      const h = parseFloat(cs.getPropertyValue('--minimap-max-h')) || fallback.h;
      setBudget((b) => (b.w === w && b.h === h ? b : { w, h }));
    };
    read();
    addEventListener('resize', read);
    return () => removeEventListener('resize', read);
  }, [el, fallback.w, fallback.h]);
  return budget;
}

export function Minimap() {
  const phone = usePhoneLayout();
  const view = useGame((s) => s.views?.[s.seat] ?? null);
  const [collapsed, setCollapsed] = useState(readCollapsed);
  const [open, setOpen] = useState(false);
  const panel = useRef<HTMLElement>(null);
  const overview = useRef<HTMLDivElement>(null);
  const opener = useRef<HTMLButtonElement>(null);
  const budget = useBudget(panel, { w: 306, h: 150 });
  const [overviewBudget, setOverviewBudget] = useState({ w: 358, h: 360 });

  // Publish the panel's height so the turn dock can stack above it (styles.css, "Minimap").
  useLayoutEffect(() => {
    const el = panel.current, game = el?.closest<HTMLElement>('.game');
    if (!el || !game) return;
    const measure = () => game.style.setProperty('--minimap-height', `${Math.round(el.getBoundingClientRect().height)}px`);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => { observer.disconnect(); game.style.removeProperty('--minimap-height'); };
  }, [phone, !!view]); // eslint-disable-line react-hooks/exhaustive-deps

  useLayoutEffect(() => {
    if (!open) return;
    const size = () => setOverviewBudget({ w: Math.min(innerWidth - 48, 640), h: Math.max(160, innerHeight * 0.55) });
    size();
    overview.current?.querySelector<HTMLElement>('.minimap-close')?.focus({ preventScroll: true });
    addEventListener('resize', size);
    return () => removeEventListener('resize', size);
  }, [open]);
  useEffect(() => { if (!phone) setOpen(false); }, [phone]);

  if (!view) return null;
  const label = 'Minimap of the known world. Click to move the map there; drag to pan.';

  if (phone) {
    const close = () => { setOpen(false); opener.current?.focus({ preventScroll: true }); };
    return <>
      <button ref={opener} type="button" className="minimap-button" data-overlay-slot="map-overview" aria-label="World map overview" aria-haspopup="dialog" aria-expanded={open} onClick={() => setOpen(true)}>
        {MAP_ICON}
        <span>Map</span>
      </button>
      {open && <div className="minimap-overview-scrim" data-game-overlay onPointerDown={(e) => { if (e.target === e.currentTarget) close(); }}>
        <div ref={overview} className="minimap-overview" role="dialog" aria-modal="true" aria-label="World map" onKeyDown={(e) => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); } }}>
          <div className="minimap-overview-head">
            <b>World map</b>
            <button type="button" className="btn quiet minimap-close" onClick={close}>Close</button>
          </div>
          <p className="minimap-overview-note">Tap a spot to go there.</p>
          <MinimapCanvas view={view} maxW={overviewBudget.w} maxH={overviewBudget.h} label="Map of the known world. Tap to move the map there." onPick={(x, y, done) => {
            centerOnWorld(x, y);
            if (done) close();
          }} />
          <Legend />
        </div>
      </div>}
    </>;
  }

  const toggle = () => { const next = !collapsed; setCollapsed(next); writeCollapsed(next); };
  return (
    <section ref={panel} className="panel minimap" data-overlay-slot="minimap" data-collapsed={collapsed} aria-label="Minimap">
      <button type="button" className="minimap-toggle" aria-expanded={!collapsed} aria-label={collapsed ? 'Show minimap' : 'Hide minimap'} onClick={toggle}>
        {collapsed ? <>{MAP_ICON}<span>Show map</span></> : <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 6l4 4 4-4" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" /></svg>}
      </button>
      {!collapsed && <MinimapCanvas view={view} maxW={budget.w} maxH={budget.h} label={label} onPick={(x, y, done) => {
        centerOnWorld(x, y);
        if (done) returnFocusToMap();
      }} />}
    </section>
  );
}
