// The 3D map (opt-in until it is finished; the 2D map stays as an option). It reads the same store
// and issues the same commands as the 2D map (commands.ts), so orders, the minimap, the advisor's
// ring and jump-to work on either map.
import { useEffect, useMemo, useRef, useState } from 'react';
import type { Hex } from '../types';
import { isEliminated, useGame } from '../store';
import { distance, hexEq, tileIndex } from '../hex';
import { enemyAt, moveSelected, peacefulAt, peacefulRangedAt, rangedEnemyAt, rangedSelected, selectedOwnUnit, warQueuedNow } from '../commands';
import { pointerIntent, previewDestination } from '../map/intent';
import { chariotTerrainNote, planTurns, previewRoute, routeDangerText, routeRiskText, type PathStep } from '../path';
import { useBattlePreview } from '../map/battlePreview';
import { ForecastCard } from '../ui/ForecastCard';
import { DualForecastCard } from '../ui/DualForecastCard';
import { PeaceHint } from '../ui/PeaceHint';
import { TileCard } from '../ui/TileCard';
import type { CardAnchor } from '../ui/usePlacedCard';
import { civilianCaptureRace, reinforcementNote } from '../ui/forecastWarnings';
import { empireTitle } from '../empires';
import { relationWith } from '../diplomacy';
import { UNIT_ACTIONS, orderUnit } from '../orders';
import { registerMapCamera, notifyMapCamera, takeMapHandoff } from '../map/camera';
import { useLandSound } from '../map/useLandSound';
import { setMapProjector } from '../map/screenPos';
import { gameplayFocusAllowed, gameplayKeyAllowed } from '../gameplayKeys';
import { feel, useCombatFx } from '../feel';
import { cancelBattleSounds, playBattleSounds } from '../feel/sound';
import { battleSoundCues } from '../feel/battleSound';
import { motionStill } from '../ui/motion';
import { useReducedMotion } from '../ui/useReducedMotion';
import { MapScene } from './scene';
import { useMapView } from './mapPrefs';
import { useMapLens } from '../map/useMapLens';
import { LensSwitcher } from '../map/LensSwitcher';
import { CompassRose } from '../map/CompassRose';
import { compassCovered, COMPASS_R } from '../map/compass';
import { drawLens, lensWash } from '../map/drawLens';
import { drawYieldIcons } from '../map/yieldIcons';
import { overlayEmpty } from '../map/overlays';
import { isBarbarian } from '../barbarians';
import { isColoniaFounder } from '../goalCities';
import { itemName } from '../data';
import { drawMarks2d, drawRoute3d, type Marks2dInput } from './marks2d';
import { cityBuildsFor, cityFocusesFor, pendingMarksFor } from '../map/orderMarks';
import { alarmedCities } from '../siege';
import { useCityAlarms } from '../ui/useCityAlarms';
import type { CityView } from '../types';
import { PX } from './world';
import './map3d.css';

export default function Map3DView() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const marksRef = useRef<HTMLCanvasElement>(null);
  const sceneRef = useRef<MapScene | null>(null);
  const views = useGame((s) => s.views);
  const seat = useGame((s) => s.seat);
  const selection = useGame((s) => s.selection);
  const orders = useGame((s) => s.orders[s.seat]);
  const mode = useGame((s) => s.mode);
  const focus = useGame((s) => s.focus);
  const pingMode = useGame((s) => s.pingMode);
  const still = motionStill(useReducedMotion(), useCombatFx((s) => s.enabled));
  const stillRef = useRef(still);
  stillRef.current = still;
  useEffect(() => { if (sceneRef.current) sceneRef.current.still = still; }, [still]);
  const anim = useGame((s) => s.anim);
  useEffect(() => {
    if (!anim?.plan || still) return;
    playBattleSounds(battleSoundCues(anim.plan), performance.now() - anim.start);
    return cancelBattleSounds;
  }, [anim, still]);
  const view = views?.[seat] ?? null;
  const [hover, setHover] = useState<Hex | null>(null);
  const client = useGame((s) => s.client);
  const forecastTick = useGame((s) => s.forecastTick);
  const wrapRef = useRef<HTMLDivElement>(null);
  /** Docked panels the hover cards keep clear of (map-relative px), sampled like the 2D map does. */
  const panels = useRef<{ left: number; top: number; right: number; bottom: number }[]>([]);
  // The 2D / 3D switch lives on the minimap; when the sidebar has no room for the minimap (the 1000x600 laptop with a
  // unit selected), a "2D map" chip sits beside the Lens control instead, so the switch is always one click away.
  const [flatChip, setFlatChip] = useState<{ left: number; bottom: number } | null>(null);
  useEffect(() => {
    const sample = () => {
      const wrap = wrapRef.current;
      if (!wrap) return;
      const o = wrap.getBoundingClientRect();
      const tab = document.querySelector('.minimap .map-view-toggle')?.getBoundingClientRect();
      const shown = !!tab && tab.width > 0 && tab.bottom <= window.innerHeight && tab.top >= 0;
      const lensBox = wrap.querySelector('.map-lenses')?.getBoundingClientRect();
      const next = shown || !lensBox ? null : { left: Math.round(lensBox.right - o.left + 8), bottom: Math.round(o.bottom - lensBox.bottom) };
      setFlatChip((old) => (old?.left === next?.left && old?.bottom === next?.bottom ? old : next));
      panels.current = [...document.querySelectorAll('.game .panel, .game .vp, .game .report, .map-lenses, .advisor-card, .topbar')].map((e) => {
        const r = e.getBoundingClientRect();
        return { left: r.left - o.left, top: r.top - o.top, right: r.right - o.left, bottom: r.bottom - o.top };
      });
    };
    sample();
    const t = window.setInterval(sample, 250);
    return () => window.clearInterval(t);
  }, []);
  // The tile card waits a moment, so it doesn't flicker while the mouse sweeps (as on the 2D map).
  const [tileHoverReady, setTileHoverReady] = useState(false);
  useEffect(() => {
    setTileHoverReady(false);
    if (!hover) return;
    const t = setTimeout(() => setTileHoverReady(true), 650);
    return () => clearTimeout(t);
  }, [hover?.q, hover?.r]); // eslint-disable-line react-hooks/exhaustive-deps

  // Swapped in from the 2D map: start where it was looking, not on the capital or an old jump-to.
  const handoff = useRef<ReturnType<typeof takeMapHandoff>>(null);
  const mountFocus = useRef(focus);
  /** The last jump-to this map has acted on (the one current at mount counts: it starts there or on the handed-over centre). */
  const appliedSeq = useRef(focus?.seq ?? 0);
  const firstView = useRef(true);

  // Scene lifetime.
  // The same land and sea sound as the 2D map (it reads the camera this view registers below).
  useLandSound();
  useEffect(() => {
    firstView.current = true; // a new scene (StrictMode remounts too) is placed again by the view effect
    handoff.current = takeMapHandoff(useGame.getState().focus?.seq ?? 0);
    const canvas = canvasRef.current!;
    const sc = new MapScene(canvas);
    sc.still = stillRef.current;
    sceneRef.current = sc;
    // Tests and local shot scripts reach the scene here (the 2D map exposes `legatumMap` the same way).
    (window as unknown as { legatumMap3d?: MapScene }).legatumMap3d = sc;
    const size = () => { sc.resize(canvas.clientWidth, canvas.clientHeight); sc.placeCamera(); notifyMapCamera(); };
    size();
    const ro = new ResizeObserver(size);
    ro.observe(canvas);
    const unregister = registerMapCamera({
      viewport: () => {
        const w = canvas.clientWidth, h = canvas.clientHeight;
        const ground = 2 * sc.dist * Math.tan((sc.camera.fov * Math.PI) / 360) * sc.camera.aspect;
        return { x: sc.target.x * PX, y: sc.target.z * PX, zoom: w / Math.max(1, ground * PX), w, h };
      },
      focusSeq: () => appliedSeq.current,
      center: (x, y) => { sc.target.set(x / PX, 0, y / PX); sc.clampTarget(); sc.placeCamera(); notifyMapCamera(); },
    });
    setMapProjector((hex) => {
      const p = sc.project(hex);
      if (!p) return null;
      const r = canvas.getBoundingClientRect();
      return { x: r.left + p.x, y: r.top + p.y, radius: p.radius };
    });
    let raf = 0;
    const loop = () => {
      sc.frame();
      drawMarks();
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
      unregister();
      setMapProjector(null);
      sc.dispose();
      sceneRef.current = null;
    };
  }, []);

  // A new view (turn, seat or fog change) rebuilds what changed.
  useEffect(() => {
    const sc = sceneRef.current;
    if (!sc || !view) return;
    sc.setView(view);
    if (firstView.current) {
      firstView.current = false;
      const from = handoff.current;
      const home = view.cities.find((c) => c.owner === view.me && c.is_capital) ?? view.cities.find((c) => c.owner === view.me) ?? view.units.find((u) => u.owner === view.me);
      if (from) { sc.target.set(from.x / PX, 0, from.y / PX); sc.clampTarget(); sc.placeCamera(); }
      else if (mountFocus.current) sc.centerOn(mountFocus.current.hex);
      else if (home) sc.centerOn(home.pos);
      notifyMapCamera();
    }
  }, [view]);
  // The turn's moves and fighting play on the 3D figures (after the new view built them), heard on the 2D map's clock.
  useEffect(() => { sceneRef.current?.setAnim(still ? null : anim); }, [anim, still]);

  // Jump-to requests (turn report lines, the advisor, Cities seen).
  useEffect(() => {
    if (!focus || focus === mountFocus.current || !sceneRef.current) return;
    sceneRef.current.centerOn(focus.hex); appliedSeq.current = focus.seq; notifyMapCamera();
  }, [focus]);

  // Overlays: hover ring, selection ring and the selected unit's route preview.
  const ownSel = useMemo(() => {
    if (!view || view.finished || isEliminated(view) || selection?.kind !== 'unit') return null;
    const u = view.units.find((x) => x.id === selection.id);
    return u && u.owner === view.me ? u : null;
  }, [view, selection]);
  const pendingMove = useMemo(() => {
    if (!ownSel) return null;
    const o = (orders ?? []).find((x) => UNIT_ACTIONS.has(x.type) && orderUnit(x) === ownSel.id);
    if (o?.type === 'MoveUnit' || o?.type === 'MoveUnitPath') return o.to;
    return o ? null : ownSel.goto;
  }, [ownSel, orders]);
  // Lenses and overlays: the same switcher, choices and tiles as the 2D map; the wash lies on the 3D ground.
  const { lens, setLens, settleUnit, sites, lensTiles, overlayTiles } = useMapLens(view, ownSel, seat, orders ?? []);
  const lensRef = useRef({ lensTiles, yieldTiles: overlayTiles.yields, settle: false });
  lensRef.current = { lensTiles, yieldTiles: overlayTiles.yields, settle: lens === 'settle' };
  useEffect(() => {
    const sc = sceneRef.current;
    if (!sc) return;
    const wash = lensWash(lensTiles, 'atlas');
    sc.setLensWash([...lensTiles, ...overlayTiles.borders].flatMap((t) => { const x = wash(t); return x ? [{ index: t.index, color: x.color, alpha: x.alpha }] : []; }));
  }, [lensTiles, overlayTiles, view]);
  const compassRef = useRef<HTMLDivElement>(null);

  const intent = view && hover && !pingMode ? pointerIntent(view, ownSel, hover, mode) : 'none';
  const previewTarget = pingMode ? null : previewDestination(intent, hover, null, pendingMove);
  const pathInfo = useMemo(() => {
    if (!view || !ownSel || !previewTarget || hexEq(previewTarget, ownSel.pos)) return null;
    const { path, risk, danger, crossed } = previewRoute(view, ownSel.pos, previewTarget, orders ?? [], ownSel.id, { avoidEnemyReach: true });
    if (!path) return { route: [] as Hex[], steps: [] as PathStep[], warning: null as string | null };
    const steps = planTurns(view, path, ownSel.pos, ownSel.max_moves, (h) => !!crossed?.some((c) => hexEq(c, h)), ownSel);
    return {
      route: [ownSel.pos, ...path.filter((h) => !hexEq(h, ownSel.pos))],
      steps,
      warning: [civilianCaptureRace(view, ownSel, previewTarget, steps.length), risk && routeRiskText(view, risk), danger && routeDangerText(danger)].filter(Boolean).join(' ') || null,
    };
  }, [view, ownSel, previewTarget, orders]);
  useEffect(() => {
    const sc = sceneRef.current;
    if (!sc || !view) return;
    const selHex = selection?.kind === 'unit' ? view.units.find((u) => u.id === selection.id)?.pos
      : selection?.kind === 'city' ? view.cities.find((c) => c.id === selection.id)?.pos : null;
    sc.setOverlays({ hover, selected: selHex ?? null, route: [] });
  }, [view, hover, pathInfo, selection]);

  // Hover cards: the same battle forecast, Peace hint and tile card as the 2D map.
  const { blockedShot, forecast, assault, shotWhy, shotForecast } = useBattlePreview(client, view, ownSel, pingMode ? null : hover, mode, orders, forecastTick);
  const routeWarningAt = (h: Hex) => (previewTarget && hexEq(h, previewTarget) ? pathInfo?.warning ?? undefined : undefined);
  const peaceHere = (() => {
    if (!hover || !view || !ownSel?.military) return null;
    if (intent === 'peaceful') return peacefulAt(view, hover);
    if (mode !== 'ranged' || distance(ownSel.pos, hover) > ownSel.range || rangedEnemyAt(view, hover)) return null;
    const peace = peacefulRangedAt(view, hover);
    return peace && !warQueuedNow(view, orders ?? [], peace.team) ? peace : null;
  })();
  const cardAnchor: CardAnchor | null = (() => {
    const sc = sceneRef.current;
    const at = hover && sc ? sc.project(hover) : null;
    if (!at) return null;
    const avoid = [...panels.current];
    const u = ownSel && sc ? sc.project(ownSel.pos, 0.15) : null;
    if (u) avoid.unshift({ left: u.x - at.radius, top: u.y - at.radius * 2, right: u.x + at.radius, bottom: u.y + at.radius });
    return { x: at.x, y: at.y, r: at.radius, avoid };
  })();
  const routeRef = useRef<{ route: Hex[]; steps: PathStep[]; attack: boolean } | null>(null);
  routeRef.current = pathInfo && pathInfo.route.length > 1 && view && previewTarget
    ? { route: pathInfo.route, steps: pathInfo.steps, attack: !!ownSel?.military && !!enemyAt(view, previewTarget) } : null;
  const hoverTile = view && hover ? view.tiles[tileIndex(hover, view.width, view.height)] : null;

  // City banners and unit marks: the 2D map's sprites on a flat canvas over the scene, drawn every frame.
  const orderErrors = useGame((s) => s.errors[s.seat]);
  const cityAlarms = useCityAlarms();
  const marksData = useRef<Omit<Marks2dInput, 'view' | 'cityAt' | 'unitAt' | 'far'> | null>(null);
  marksData.current = useMemo(() => {
    const focuses = cityFocusesFor(view, orders ?? []);
    return {
      marks: pendingMarksFor(view, orders ?? [], orderErrors),
      focusOf: (c: CityView) => focuses.get(c.id) ?? null,
      builds: cityBuildsFor(view, orders ?? []),
      siege: alarmedCities(cityAlarms),
      selectedCity: selection?.kind === 'city' ? selection.id : null,
    };
  }, [view, orders, orderErrors, cityAlarms, selection]);
  const drawMarks = () => {
    const sc = sceneRef.current, cv = marksRef.current, data = marksData.current;
    const v = useGame.getState().views?.[useGame.getState().seat];
    if (!sc || !cv || !data || !v) return;
    const w = cv.clientWidth, h = cv.clientHeight, dpr = Math.min(window.devicePixelRatio || 1, 2);
    if (cv.width !== Math.round(w * dpr) || cv.height !== Math.round(h * dpr)) { cv.width = Math.round(w * dpr); cv.height = Math.round(h * dpr); }
    const ctx = cv.getContext('2d');
    if (!ctx) return;
    const known = (c: CityView) => v.tiles[tileIndex(c.pos, v.width, v.height)]?.explored;
    drawMarks2d(ctx, w, h, dpr, {
      ...data, view: v, far: sc.dist > 14,
      cityAt: (c) => (known(c) ? sc.project(c.pos, 0.34) : null),
      unitAt: (u) => sc.unitHead(u.id),
    });
    const rt = routeRef.current;
    if (rt) drawRoute3d(ctx, rt.route.map((hx) => sc.project(hx, 0.02)), rt.steps, rt.attack, performance.now());
    const { lensTiles: lt, yieldTiles, settle } = lensRef.current;
    if (lt.length || yieldTiles.length) {
      // One hex size for the whole view (the one under the camera's target): chips and icons stay one size, as on the 2D map.
      const hexPx = Math.max(6, sc.hexPx());
      const at = (i: number) => sc.projectIndex(i);
      const cam = { x: 0, y: 0, zoom: 1 };
      if (yieldTiles.length) drawYieldIcons(ctx, yieldTiles, cam, w, h, v.width, dpr, 'atlas', settle, [], at, hexPx);
      if (lt.length) drawLens(ctx, lt, cam, w, h, v.width, dpr, 'atlas', 'labels', new Map(), panelRects(), () => 0, { at, hexPx, yieldsOn: yieldTiles.length > 0 });
    }
    // The compass: north is always up on this map, so it rests in the lower-left corner, hidden when a panel covers it.
    const cp = compassRef.current;
    if (cp) {
      const x = 24 + COMPASS_R, y = h - 24 - COMPASS_R;
      cp.style.transform = `translate(${x}px, ${y}px)`;
      cp.classList.toggle('covered', compassCovered(x, y, panelRects()));
    }
  };
  const panelRects = () => panels.current.map((p) => ({ x: p.left, y: p.top, w: p.right - p.left, h: p.bottom - p.top }));

  // ---- input ----
  const drag = useRef<{ x: number; y: number; ground: { x: number; z: number } | null; moved: boolean } | null>(null);
  const local = (e: { clientX: number; clientY: number }) => {
    const r = canvasRef.current!.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  const clickHex = (h: Hex) => {
    const s = useGame.getState();
    const v = s.views?.[s.seat];
    if (!v || tileIndex(h, v.width, v.height) < 0) return;
    if (s.pingMode) { s.sendPing(h); return; }
    if (s.mode === 'ranged' && !v.finished && !isEliminated(v)) { rangedSelected(h); return; }
    if (s.mode === 'move' && !v.finished && !isEliminated(v)) { moveSelected(h); return; }
    const here = v.units.filter((u) => hexEq(u.pos, h));
    const city = v.cities.find((c) => hexEq(c.pos, h));
    const unitCands = (own: boolean) => here.filter((u) => (u.owner === v.me) === own)
      .sort((a, b) => Number(b.military) - Number(a.military)).map((u) => ({ kind: 'unit' as const, id: u.id }));
    const cands: { kind: 'unit' | 'city'; id: number }[] = [...unitCands(true), ...(city ? [{ kind: 'city' as const, id: city.id }] : []), ...unitCands(false)];
    if (!cands.length) { s.select(null); return; }
    const cur = s.selection;
    const idx = cur ? cands.findIndex((c) => c.kind === cur.kind && c.id === cur.id) : -1;
    s.select(cands[(idx + 1) % cands.length]);
    feel.select();
  };

  const onPointerDown = (e: React.PointerEvent) => {
    canvasRef.current?.focus({ preventScroll: true });
    if (e.button !== 0) return;
    const p = local(e);
    const g = sceneRef.current?.groundAt(p.x, p.y);
    drag.current = { x: p.x, y: p.y, ground: g ? { x: g.x, z: g.z } : null, moved: false };
    e.currentTarget.setPointerCapture(e.pointerId);
  };
  const onPointerMove = (e: React.PointerEvent) => {
    const sc = sceneRef.current;
    if (!sc) return;
    const p = local(e);
    const d = drag.current;
    if (d) {
      if (!d.moved && Math.hypot(p.x - d.x, p.y - d.y) > 4) d.moved = true;
      if (d.moved && d.ground) {
        const g = sc.groundAt(p.x, p.y);
        if (g) {
          sc.target.x -= g.x - d.ground.x; sc.target.z -= g.z - d.ground.z;
          sc.clampTarget(); sc.placeCamera(); notifyMapCamera();
        }
        setHover(null);
        return;
      }
    }
    const h = sc.pick(p.x, p.y);
    setHover((old) => (old && h && hexEq(old, h) ? old : h));
  };
  const onPointerUp = (e: React.PointerEvent) => {
    const d = drag.current;
    drag.current = null;
    if (e.currentTarget.hasPointerCapture?.(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    if (!d || d.moved) return;
    const h = sceneRef.current?.pick(d.x, d.y);
    if (h) clickHex(h);
  };
  const onContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    const p = local(e);
    const h = sceneRef.current?.pick(p.x, p.y);
    if (h && selectedOwnUnit()) moveSelected(h);
  };

  // Wheel zoom toward the cursor (non-passive, so the page never scrolls).
  useEffect(() => {
    const c = canvasRef.current!;
    const wheel = (e: WheelEvent) => {
      e.preventDefault();
      const sc = sceneRef.current;
      if (!sc) return;
      const p = local(e);
      const before = sc.groundAt(p.x, p.y);
      const k = e.ctrlKey ? 0.012 : 0.0012;
      sc.dist = Math.min(34, Math.max(1.6, sc.dist * Math.exp(e.deltaY * k)));
      sc.placeCamera();
      const after = sc.groundAt(p.x, p.y);
      if (before && after) { sc.target.x += before.x - after.x; sc.target.z += before.z - after.z; sc.clampTarget(); sc.placeCamera(); }
      notifyMapCamera();
    };
    c.addEventListener('wheel', wheel, { passive: false });
    return () => c.removeEventListener('wheel', wheel);
  }, []);

  // Keyboard panning, as on the 2D map.
  useEffect(() => {
    const keys = new Set<string>();
    let raf = 0;
    const panKeys = ['w', 'a', 's', 'd', 'arrowup', 'arrowdown', 'arrowleft', 'arrowright'];
    const stop = () => { keys.clear(); cancelAnimationFrame(raf); };
    const tick = () => {
      const sc = sceneRef.current;
      if (!sc || !gameplayFocusAllowed(document.activeElement, 'arrowup')) { stop(); return; }
      const sp = sc.dist * 0.015;
      if (keys.has('w') || keys.has('arrowup')) sc.target.z -= sp;
      if (keys.has('s') || keys.has('arrowdown')) sc.target.z += sp;
      if (keys.has('a') || keys.has('arrowleft')) sc.target.x -= sp;
      if (keys.has('d') || keys.has('arrowright')) sc.target.x += sp;
      sc.clampTarget(); sc.placeCamera(); notifyMapCamera();
      if (keys.size) raf = requestAnimationFrame(tick);
    };
    const down = (e: KeyboardEvent) => {
      if (!gameplayKeyAllowed(e)) return;
      const k = e.key.toLowerCase();
      if (!panKeys.includes(k)) return;
      e.preventDefault();
      if (!keys.size) raf = requestAnimationFrame(tick);
      keys.add(k);
    };
    const up = (e: KeyboardEvent) => { keys.delete(e.key.toLowerCase()); if (!keys.size) stop(); };
    window.addEventListener('keydown', down);
    window.addEventListener('keyup', up);
    window.addEventListener('blur', stop);
    return () => { stop(); window.removeEventListener('keydown', down); window.removeEventListener('keyup', up); window.removeEventListener('blur', stop); };
  }, []);

  return (
    <div ref={wrapRef} className="map-wrap map3d-wrap">
      <canvas
        ref={canvasRef}
        tabIndex={0}
        data-game-map
        data-overlay-slot="map-viewport"
        aria-label="3D world map. Press M for the 2D map."
        className="map-canvas map3d-canvas"
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={() => setHover(null)}
        onContextMenu={onContextMenu}
      />
      <canvas ref={marksRef} className="map3d-marks" aria-hidden="true" />
      <CompassRose ref={compassRef} />
      {flatChip && <button type="button" className="map3d-flat-chip" style={{ left: flatChip.left, bottom: flatChip.bottom }}
        aria-label="Show the 2D map (M)" title="Show the 2D map (M)" onClick={() => useMapView.getState().set('2d')}>2D map</button>}
      <LensSwitcher players={view?.players ?? []} me={view?.me} visibleBarbarians={!!view?.units.some((u) => isBarbarian(u.owner))} supply={view?.supply} era={view?.research.era}
        lens={lens} empty={!lensTiles.length}
        overlayEmpty={{ resources: !!view && overlayEmpty(view, 'resources', overlayTiles), borders: !!view && overlayEmpty(view, 'borders', overlayTiles) }}
        collapsed={!!selection} onChange={setLens} sites={sites} settlerId={settleUnit?.id ?? null}
        coloniaFounder={!!view && !!settleUnit && isColoniaFounder(view, settleUnit.id)}
        founderName={settleUnit ? itemName({ kind: 'Unit', unit: settleUnit.kind }, view?.players.find((p) => p.id === settleUnit.owner)?.faction) : undefined}
        onInspect={(hex) => { useGame.getState().jumpTo(hex); setHover(null); }} />
      {cardAnchor && view && peaceHere && <PeaceHint anchor={cardAnchor} team={empireTitle(view, peaceHere.team)} mouse allied={relationWith(view, peaceHere.team)?.state === 'Alliance'} />}
      {cardAnchor && view && ownSel && hover && forecast && (shotForecast
        ? <DualForecastCard shot={shotForecast} charge={forecast} anchor={cardAnchor} view={view} unit={ownSel} target={hover} routeWarning={routeWarningAt(hover)} />
        : <ForecastCard forecast={forecast} anchor={cardAnchor} ranged={forecast.ranged} view={view} unit={ownSel} target={hover} routeWarning={forecast.ranged ? undefined : routeWarningAt(hover)} bowNote={shotWhy ?? undefined} assault={assault} />)}
      {/* One card at a time: a forecast or the Peace hint is the decision here, so the tile card waits. */}
      {cardAnchor && view && hover && !forecast && !peaceHere && tileHoverReady && hoverTile?.explored && (
        <TileCard view={view} hex={hover} anchor={cardAnchor} blockedReason={blockedShot || undefined} routeWarning={ownSel ? routeWarningAt(hover) : undefined}
          reinforcement={ownSel ? reinforcementNote(view, ownSel, hover) ?? undefined : undefined}
          note={ownSel ? chariotTerrainNote(view, ownSel, hover) : undefined} />
      )}
    </div>
  );
}
