import { retirePreview, swarmPreview, wildPreview } from '../goalActions';
import { useCelebration } from '../celebrate/MomentLane';
import { drawCounterFlights, EFFECT_MS, FLIGHT_MS, type MapCelebration } from '../celebrate/mapEffect';
import { visibleTilePoint } from '../celebrate/visibleTile';
import { isExploring } from '../automation';
import { attacksInMelee, itemName, tileRingPercent } from '../data';
import { yieldsText } from '../cityProgress';
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { isEliminated, useGame } from '../store';
import type { AlongsideAttack, BattleForecast, CityAssaultForecast, CityBuildFocus, Hex, PlayerView, ProductionItem } from '../types';
import { cityAutomationFocus, cityAutomationPreview } from '../automation';
import { distance, hexEq, hexToPixel, pixelToHex, tileIndex, within } from '../hex';
import { planTurns, previewRoute, roadBonusNote, routeTurnsText, routeRiskText, routeDangerText, isCalmRouteNote, type PathStep, type RouteRisk } from '../path';
import { draw, HEX_SIZE, perf, screenToWorld, terrainStats, worldBounds, ambientClipRects, hoverClipBoxes, idleClipBoxes, type Camera, type SelectionRing, type ImproveOverlay, type PendingMark, type TerritoryOverlay } from './render';
import { fontEpoch } from './sprites';
import { onIconLoad } from '../icons';
import { onMapFeatureLoad } from './mapFeatures';
import { onCityArtLoad } from './cityArt';
import { onImprovementArtLoad } from './improvementArt';
import { onLandmarkArtLoad } from './landmarks';
import { onWonderArtLoad } from './wonderArt';
import { cancelSelected, promptWar, enemyAt, peacefulAt, peacefulRangedAt, rangedEnemyAt, warQueuedNow, foundCitySelected, sleepSelected, skipSelected, moveSelected, rangedSelected, selectedOwnUnit, toggleRangedMode } from '../commands';
import { pointerIntent, previewDestination } from './intent';
import { foundCityBlocker, foundCityUnitBlocker, isFounderCandidate, reservedSettlements, settlementScores, settlementSite } from '../settlement';
import { buildLensTiles, resolveLens, type MapLens } from './lenses';
import { buildOverlayTiles, overlayEmpty, useOverlays } from './overlays';
import { settleTileNote, withProxySettler } from './settleAdvice';
import { LensSwitcher } from './LensSwitcher';
import { isBarbarian } from '../barbarians';
import { SettlementCard, SettlementDetails } from './SettlementCard';
import { arrowAt, battleCues, cuesAt, ARROW_FADE_MS, ARROW_HOLD_MS, CUE_FADE_MS, CUE_HOLD_MS, type MapCue, type ReportArrow } from './cues';
import { UNIT_ACTIONS, orderUnit } from '../orders';
import { ForecastCard, ForecastSummary, forecastTargetName } from '../ui/ForecastCard';
import { DualForecastCard } from '../ui/DualForecastCard';
import { civilianCaptureRace, reinforcementNote } from '../ui/forecastWarnings';
import type { CardAnchor } from '../ui/usePlacedCard';
import { PeaceHint } from '../ui/PeaceHint';
import { empireTitle } from '../empires';
import { bannerCityAt, bannerTooltip } from './bannerTooltip';
import { TileCard } from '../ui/TileCard';
import { mapPlayerColor } from './style';
import { shotBlockReason } from '../los';
import { CompassRose } from './CompassRose';
import { placeCompass as compassPlace, type Rect } from './compass';
import { gameplayFocusAllowed, gameplayKeyAllowed } from '../gameplayKeys';
import { useReducedMotion } from '../ui/useReducedMotion';
import { usePhoneLayout } from '../ui/usePhoneLayout';
import { motionStill } from '../ui/motion';
import { feel, useCombatFx } from '../feel';
import { captureTime, fxLive, fxUnits, stepPos, type FxGhost } from './combatFx';
import { IDLE_FPS } from './idleLife';
import { hasVisibleSmoulder } from './smoulder';
import { holdAt, marchOf, momentTime, momentsLive, type March } from './momentFx';
import { atWar, relationWith } from '../diplomacy';

import { emptyGesture, gestureStep, isDoubleTap, LONG_PRESS_MS, zoomCamera, type GestureEvent, type Point, type Tap } from './gestures';
import { sameOrderContext, tapOrder, type TapOrder } from './tapOrders';
import { coord, setUnitOrder } from '../orders';
import { notifyMapCamera, registerMapCamera } from './camera';
import { alarmedCities } from '../siege';
import { useCityAlarms } from '../ui/useCityAlarms';
import { setBorderVariant, type BorderVariant } from './borderVariant';
import { aircraftAsUnit, airOverlayFor, pickerCandidates } from '../airWing';
import { pendingProduction, productionQueue, setAirOrder, toggleBuyTile } from '../orders';
import { tilePurchasePreview } from '../ui/productionPurchases';

/** Whether a field the draw loop reads changed between two renders (by identity; the effects flag by value). */
export function frameChanged<T extends Record<string, unknown>>(prev: T, next: T): boolean {
  for (const key of Object.keys(next)) {
    const a = prev[key], b = next[key];
    if (key === 'combatFx') {
      if ((a as { enabled: boolean } | undefined)?.enabled !== (b as { enabled: boolean }).enabled) return true;
    } else if (!Object.is(a, b)) return true;
  }
  return false;
}

/** The fields that follow the tile under the pointer: the hover mark and the selected unit's route preview. */
const HOVER_FIELDS = new Set(['hover', 'hoverSelect', 'pathInfo', 'pathAttack']);

/** Whether every field that changed between two renders is one of the hover's (and at least one did). */
export function hoverOnlyChange<T extends Record<string, unknown>>(prev: T, next: T): boolean {
  let hover = false;
  for (const key of Object.keys(next)) {
    const one = { [key]: prev[key] }, other = { [key]: next[key] };
    if (!frameChanged(one, other)) continue;
    if (!HOVER_FIELDS.has(key)) return false;
    hover = true;
  }
  return hover;
}

interface Hover {
  hex: Hex;
  sx: number;
  sy: number;
}

export function MapCanvas({ inspectRequest }: { inspectRequest?: Hex | null }) {
  const phone = usePhoneLayout();
  const reducedMotion = useReducedMotion();
  const combatFx = useCombatFx();
  const still = motionStill(reducedMotion, combatFx.enabled);
  const celebration = useCelebration();
  const celebrationTargets = useRef<MapCelebration['targets']>({});
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const flightRef = useRef<HTMLCanvasElement>(null);
  const flightSize = useRef({ w: 800, h: 600, x: 0, y: 0 });
  const raceRect = useRef<DOMRect | null>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const measure = () => {
      const rect = canvasRef.current?.getBoundingClientRect();
      celebrationTargets.current = {};
      raceRect.current = document.querySelector('.right-sidebar')?.getBoundingClientRect() ?? null;
      if (!rect || !celebration.active) return;
      for (const { counter } of celebration.active.deltas) {
        const target = document.querySelector(`[data-celebration-counter="${counter}"]`)?.getBoundingClientRect();
        if (target?.width && target.height) celebrationTargets.current[counter] = { x: target.x + target.width / 2 - rect.x, y: target.y + target.height / 2 - rect.y };
      }
      dirty.current = true;
    };
    measure();
    const observer = new ResizeObserver(measure);
    if (canvasRef.current) observer.observe(canvasRef.current);
    const race = document.querySelector('.right-sidebar'); if (race) observer.observe(race);
    return () => observer.disconnect();
  }, [celebration.active]);
  const compassRef = useRef<HTMLDivElement>(null);
  const panels = useRef<Rect[]>([]);
  const labels = useRef<Rect[]>([]);
  const banners = useRef<Rect[]>([]);
  const bannerHits = useRef<ReturnType<typeof draw>['bannerHits']>([]);
  const liftedHits = useRef<ReturnType<typeof draw>['liftedHits']>([]);
  const cam = useRef<Camera>({ x: 0, y: 0, zoom: 0.9 });
  const size = useRef({ w: 800, h: 600, dpr: 1 });
  const dirty = useRef(true);
  const hoverDirty = useRef(false);
  const focusAnimation = useRef(0);
  const [hover, setHover] = useState<Hover | null>(null);
  const [dragTarget, setDragTarget] = useState<Hex | null>(null);
  const [tileHoverReady, setTileHoverReady] = useState(false);
  const gesture = useRef(emptyGesture());
  const holdTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastTap = useRef<Tap | null>(null);
  const lastPointerType = useRef('mouse');
  const [tapPreview, setTapPreview] = useState<TapOrder | null>(null);
  const [touchInfo, setTouchInfo] = useState<{ hex: Hex; context: boolean; placement: 'top' | 'bottom' } | null>(null);
  const inspectorRef = useRef<HTMLElement>(null);
  const bannerTipRef = useRef<HTMLDivElement>(null);

  const views = useGame((s) => s.views);
  const seat = useGame((s) => s.seat);
  const scenarioTargets = useGame((s) => s.scenario?.status.targets);
  const cityAlarms = useCityAlarms();
  const siege = useMemo(() => alarmedCities(cityAlarms), [cityAlarms]);
  const selection = useGame((s) => s.selection);
  const orders = useGame((s) => s.orders[s.seat]);
  const orderErrors = useGame((s) => s.errors[s.seat]);
  const mode = useGame((s) => s.mode);
  const anim = useGame((s) => s.anim);
  const focus = useGame((s) => s.focus);
  const client = useGame((s) => s.client);
  const forecastTick = useGame((s) => s.forecastTick);
  const report = useGame((s) => s.report);
  const territory = useGame((s) => s.territory);
  // Battle results of the turn just resolved, shown on the map for a few seconds.
  const cues = useMemo(() => {
    const v = views?.[seat];
    if (!report || !v) return null;
    const list = battleCues(report.events[seat] ?? [], v);
    return list.length ? { list, start: performance.now() } : null;
  }, [report, seat]); // eslint-disable-line react-hooks/exhaustive-deps
  // A clicked combat line in the turn report: an arrow from the attacker, for a few seconds.
  const reportArrow = useGame((s) => s.reportArrow);
  const arrow = useMemo(() => (reportArrow ? { from: reportArrow.from, to: reportArrow.to, start: performance.now() } : null), [reportArrow]);
  const drawnArrow = useRef<ReportArrow | null>(null);
  const busy = useGame((s) => s.busy);
  const view: PlayerView | null = views ? views[seat] : null;
  // A goal action, while it is weighed or ordered: the Great Migration's shrinking cities and released tiles,
  // the warbands's cities and camps. A veteran's retirement marks the valid Agoge cities (below).
  const swarmOpen = useGame((s) => s.goalDialog === 'Swarm' || !!(s.orders[s.seat] ?? []).find((o) => o.type === 'Action' && o.action.action === 'Swarm'));
  const wildOpen = useGame((s) => s.goalDialog === 'CallTheWild' || !!(s.orders[s.seat] ?? []).find((o) => o.type === 'Action' && o.action.action === 'CallTheWild'));
  const retireWeighing = useGame((s) => s.goalDialog === 'RetireVeteran');

  const selUnit = useMemo(() => {
    if (!view || selection?.kind !== 'unit') return null;
    return view.units.find((u) => u.id === selection.id) ?? null;
  }, [view, selection]);
  const ownSel = selUnit && view && !view.finished && !isEliminated(view) && selUnit.owner === view.me ? selUnit : null;
  const goalPreview = useMemo(() => {
    if (!view || view.finished) return null;
    if (swarmOpen) return swarmPreview(view);
    if (wildOpen) return wildPreview(view);
    return retirePreview(view, ownSel, retireWeighing);
  }, [swarmOpen, wildOpen, retireWeighing, view, ownSel]);
  const [lensChoices, setLensChoices] = useState<Record<number, MapLens>>({});
  const [settlerLens, setSettlerLens] = useState<{ seat: number; unit: number; lens: MapLens } | null>(null);
  const selectedSettler = view && ownSel && isFounderCandidate(view, ownSel) ? ownSel : null;
  // Auto-Settle is temporary; it never overwrites a player's chosen lens.
  const lens = resolveLens(seat, selectedSettler?.id ?? null, lensChoices, settlerLens);
  // Overlays are the viewer's own and ride through every lens, auto-Settle included.
  const overlays = useOverlays();
  useEffect(() => { setSettlerLens(null); }, [seat, selectedSettler?.id]);
  const setLens = (next: MapLens) => {
    setLensChoices((choices) => ({ ...choices, [seat]: next }));
    if (selectedSettler) setSettlerLens({ seat, unit: selectedSettler.id, lens: next });
  };
  const settleUnit = selectedSettler ?? (lens === 'settle' ? view?.units.find((u) => u.owner === view.me && isFounderCandidate(view, u)) : null);
  // No Settler on the map: the lens still marks the explored map, scored for a stand-in Settler at the capital.
  const proxy = useMemo(() => view && lens === 'settle' && !settleUnit ? withProxySettler(view) : null, [view, lens, settleUnit]);
  const scoreUnit = settleUnit ?? proxy?.unit ?? null;
  const scoreView = proxy?.view ?? view;
  const reserved = useMemo(() => view && scoreUnit ? reservedSettlements(view, scoreUnit.id, orders) : [], [view, scoreUnit, orders]);
  const sites = useMemo(() => scoreView && scoreUnit && lens === 'settle'
    ? settlementScores(scoreView, scoreUnit, reserved) : [], [scoreView, scoreUnit, lens, reserved]);
  const sitesByHex = useMemo(() => new Map(sites.map((s) => [`${s.pos.q},${s.pos.r}`, s])), [sites]);
  const siteAt = (h: Hex) => sitesByHex.get(`${h.q},${h.r}`);
  const inspectedHex = touchInfo?.hex ?? (tileHoverReady ? hover?.hex : null);
  const inspectedScore = inspectedHex ? siteAt(inspectedHex) : undefined;
  const inspectedSite = useMemo(() => {
    if (!scoreView || !scoreUnit || !inspectedScore) return null;
    const site = settlementSite(scoreView, scoreUnit, inspectedScore.pos, reserved);
    return site ? { ...site, rank: inspectedScore.rank } : null;
  }, [scoreView, scoreUnit, inspectedScore, reserved]);
  const lensTiles = useMemo(() => view ? buildLensTiles(view, lens, sites) : [], [view, lens, sites]);
  const overlayTiles = useMemo(() => view ? buildOverlayTiles(view, overlays) : { yields: [], borders: [] }, [view, overlays]);
  const placeInspector = useRef<(() => void) | null>(null);
  useLayoutEffect(() => {
    const inspector = inspectorRef.current, wrap = wrapRef.current;
    placeInspector.current = null;
    if (phone || !touchInfo || !inspectedSite || !inspector || !wrap) return;
    const hex = touchInfo.hex;
    // Desktop: beside the inspected hex (never over it or its units), in a band clear of the
    // story above and the command dock below. Only when neither side fits (a cramped iPad
    // layout), fall back to the centred slot between the story and the dock.
    const above = [...document.querySelectorAll('.topbar, .first-game-hint')];
    const below = [...document.querySelectorAll('.unit-panel, .up-hints, .seatbar, .map-lenses')];
    const sides = [...document.querySelectorAll('.game .panel, .game .report, .vp')];
    const shown = (el: Element) => el.getBoundingClientRect().height > 0 && getComputedStyle(el).visibility !== 'hidden' && !el.contains(inspector);
    const measure = () => {
      const area = wrap.getBoundingClientRect(), origin = (inspector.offsetParent ?? wrap).getBoundingClientRect();
      const width = inspector.offsetWidth, r = HEX_SIZE * cam.current.zoom, at = toScreen(hex);
      const hx = area.left + at.x, hy = area.top + at.y;
      const obstacles = [...above, ...below, ...sides].filter(shown).map((el) => el.getBoundingClientRect());
      const fit = (left: number) => {
        const right = left + width;
        if (left < area.left + 8 || right > area.right - 8) return null;
        let top = area.top + 8, bottom = area.bottom - 16;
        for (const o of obstacles) {
          if (o.right <= left || o.left >= right) continue;
          if (o.bottom <= hy) top = Math.max(top, o.bottom + 12);
          else if (o.top >= hy) bottom = Math.min(bottom, o.top - 12);
          else return null;
        }
        return bottom - top >= 160 ? { left, top, bottom } : null;
      };
      const spot = hy > area.top && hy < area.bottom ? fit(hx + r + 10) ?? fit(hx - r - 10 - width) : null;
      if (spot) {
        const height = Math.min(inspector.scrollHeight, spot.bottom - spot.top);
        const y = Math.max(spot.top, Math.min(spot.bottom - height, hy - height / 2));
        inspector.dataset.beside = 'true';
        inspector.style.setProperty('--settle-inspector-left', `${spot.left - origin.left}px`);
        inspector.style.setProperty('--settle-inspector-top', `${y - origin.top}px`);
        inspector.style.setProperty('--settle-inspector-height', `${spot.bottom - y}px`);
        return;
      }
      inspector.dataset.beside = 'false';
      const left = area.left + area.width / 2 - width / 2, right = left + width;
      const inColumn = (el: Element) => { const b = el.getBoundingClientRect(); return shown(el) && b.left < right && b.right > left; };
      const top = Math.max(area.top + 8, ...above.filter(inColumn).map((el) => el.getBoundingClientRect().bottom + 12));
      const bottom = Math.min(area.bottom - 16, ...below.filter(inColumn).map((el) => el.getBoundingClientRect().top - 12));
      inspector.style.setProperty('--settle-inspector-bottom', `${area.bottom - bottom}px`);
      inspector.style.setProperty('--settle-inspector-height', `${Math.max(0, bottom - top)}px`);
    };
    measure();
    // The camera moves the hex (a recommendation jumps there): the draw loop re-places the card.
    placeInspector.current = measure;
    const resize = new ResizeObserver(measure);
    [wrap, inspector, ...above, ...below, ...sides].forEach((el) => resize.observe(el));
    return () => { resize.disconnect(); placeInspector.current = null; };
  }, [phone, touchInfo?.hex.q, touchInfo?.hex.r, !!inspectedSite]);
  const activePreview = tapPreview && ownSel && view && sameOrderContext(tapPreview, { unit: ownSel.id, seat, turn: view.turn, mode }) ? tapPreview : null;
  const clearTouch = () => { setTapPreview(null); setTouchInfo(null); };
  useEffect(() => {
    if (!inspectRequest) return;
    setHover(null);
    setTapPreview(null);
    setTouchInfo({ hex: inspectRequest, context: true, placement: 'top' });
  }, [inspectRequest]);
  useEffect(() => { setTapPreview(null); }, [selection, seat, view?.turn, mode, orders]);
  useEffect(() => {
    clearTouch();
    gesture.current = emptyGesture();
    lastTap.current = null;
    if (holdTimer.current) clearTimeout(holdTimer.current);
  }, [seat, view?.turn, busy]);
  useEffect(() => {
    const outside = (e: PointerEvent) => { if (!wrapRef.current?.contains(e.target as Node)) clearTouch(); };
    const stop = () => {
      if (holdTimer.current) clearTimeout(holdTimer.current);
      gesture.current = emptyGesture();
      lastTap.current = null;
      setDragTarget(null);
      clearTouch();
    };
    const escape = (e: KeyboardEvent) => { if (e.key === 'Escape' && gameplayKeyAllowed(e)) clearTouch(); };
    document.addEventListener('pointerdown', outside);
    window.addEventListener('keydown', escape);
    window.addEventListener('blur', stop);
    return () => {
      if (holdTimer.current) clearTimeout(holdTimer.current);
      document.removeEventListener('pointerdown', outside);
      window.removeEventListener('keydown', escape);
      window.removeEventListener('blur', stop);
    };
  }, []);

  // Path preview target: drag target, else hover (Civ shows the path on hover while a unit
  // is selected), else the unit's pending/persistent destination.
  const pendingMove = useMemo(() => {
    if (!ownSel) return null;
    const o = orders.find((x) => UNIT_ACTIONS.has(x.type) && orderUnit(x) === ownSel.id);
    if (o?.type === 'MoveUnit' || o?.type === 'MoveUnitPath') return o.to;
    return o ? null : ownSel.goto;
  }, [ownSel, orders]);

  // Hovering a tile only previews a path when the command there is a move or an attack; over the
  // player's own unit or city it shows a select ring instead (see map/intent.ts).
  const intent = view && hover ? pointerIntent(view, ownSel, hover.hex, mode) : 'none';
  const previewTarget = previewDestination(intent, hover?.hex ?? null, (mode !== 'ranged' ? activePreview?.target : null) ?? dragTarget, pendingMove);
  const pathInfo = useMemo(() => {
    if (!view || !ownSel || !previewTarget || hexEq(previewTarget, ownSel.pos)) return null;
    const { path: p, risk, danger, crossed } = previewRoute(view, ownSel.pos, previewTarget, orders, ownSel.id, { avoidEnemyReach: true });
    if (!p) return { steps: null as PathStep[] | null, unreachable: previewTarget, risk: null as RouteRisk | null, warning: null as string | null };
    // Friends the route passes through are never where a turn ends (core `path::hop`).
    const steps = planTurns(view, p, ownSel.pos, ownSel.max_moves, (h) => !!crossed?.some((c) => c.q === h.q && c.r === h.r));
    return { steps, unreachable: null, risk, warning: [civilianCaptureRace(view, ownSel, previewTarget, steps.length), risk && routeRiskText(view, risk), danger && routeDangerText(danger)].filter(Boolean).join(' ') || null };
  }, [view, ownSel, previewTarget, orders]);
  /** A route warning belongs to the tile it previews (hover or the open preview card). */
  /** The reinforcement note for a move into your own threatened, ungarrisoned city. */
  const reinforcementAt = (h: Hex | null | undefined) => (h && view && ownSel ? reinforcementNote(view, ownSel, h) ?? undefined : undefined);
  const routeWarningAt = (h: Hex | null | undefined) => (h && previewTarget && hexEq(h, previewTarget) ? pathInfo?.warning ?? undefined : undefined);
  const settlePreviewBlocker = view && ownSel && isFounderCandidate(view, ownSel) && previewTarget
    && view.tiles[tileIndex(previewTarget, view.width, view.height)]?.explored
    ? foundCityBlocker(view, previewTarget) : null;

  const pathAttack = !!(view && previewTarget && ownSel?.military && enemyAt(view, previewTarget));
  /** Ranged targeting: a unit or city of a team at Peace in range, with no War queued on them yet. */
  const rangedPeaceAt = (v: PlayerView, h: Hex) => {
    if (!ownSel || distance(ownSel.pos, h) > ownSel.range || rangedEnemyAt(v, h)) return null;
    const peace = peacefulRangedAt(v, h);
    return peace && !warQueuedNow(v, orders, peace.team) ? peace : null;
  };
  const previewPeace = view && activePreview && ownSel?.military
    ? mode !== 'ranged' ? peacefulAt(view, activePreview.target)
      : rangedPeaceAt(view, activePreview.target)
    : null;

  const pendingLines = useMemo(() => {
    if (!view) return [];
    const lines: { from: Hex; to: Hex; attack: boolean }[] = [];
    const seen = new Set<number>();
    for (const [i, o] of orders.entries()) {
      if (o.type !== 'MoveUnit' && o.type !== 'MoveUnitPath' && o.type !== 'RangedAttack') continue;
      // An order that will fail never looks accepted: no line, and the unit's mark says it needs fixing.
      if (orderErrors?.some((e) => e.index === i)) continue;
      const u = view.units.find((x) => x.id === o.unit);
      if (!u || u.id === ownSel?.id) continue;
      seen.add(u.id);
      const to = o.type === 'RangedAttack' ? o.target : o.to;
      lines.push({ from: u.pos, to, attack: o.type === 'RangedAttack' || !!enemyAt(view, to) });
    }
    for (const u of view.units) {
      if (u.owner !== view.me || !u.goto || seen.has(u.id) || u.id === ownSel?.id) continue;
      if (orders.some((o) => UNIT_ACTIONS.has(o.type) && orderUnit(o) === u.id)) continue;
      lines.push({ from: u.pos, to: u.goto, attack: u.attack_target !== undefined });
    }
    return lines;
  }, [view, orders, orderErrors, ownSel]);

  const pendingMarks = useMemo(() => {
    const m = new Map<number, PendingMark>();
    for (const u of view?.units ?? []) if (u.owner === view?.me && isExploring(u, orders)) m.set(u.id, 'explore');
    for (const [i, o] of orders.entries()) {
      const u = orderUnit(o);
      if (u === null) continue;
      if (UNIT_ACTIONS.has(o.type) && orderErrors?.some((e) => e.index === i)) { m.set(u, 'invalid'); continue; }
      const mark = (
        { MoveUnit: 'move', MoveUnitPath: 'move', SleepUnit: 'sleep', SkipUnit: 'skip', FoundCity: 'found', DisbandUnit: 'disband', RangedAttack: 'ranged', UpgradeUnit: 'upgrade', UpgradeSiege: 'upgrade' } as Record<string, PendingMark>
      )[o.type];
      if (mark) m.set(u, o.type === 'SleepUnit' && o.until_healed ? 'heal' : mark);
    }
    return m;
  }, [orders, orderErrors, view]);
  // A delegated city whose focus excludes every build waits for the player: its banner shows the red Build prompt, not "Auto".
  const cityFocuses = useMemo(() => {
    const unchosen = new Set(view ? cityAutomationPreview(view, orders)?.unchosen_cities ?? [] : []);
    return new Map<number, CityBuildFocus | null>((view?.cities ?? []).filter((c) => c.owner === view?.me)
      .map((c) => [c.id, unchosen.has(c.id) ? null : cityAutomationFocus(c.id, view!, orders)]));
  }, [view, orders]);

  // A city told what to build this turn shows it on its banner at once, not the red Build prompt until the turn resolves.
  const cityBuilds = useMemo(() => {
    const m = new Map<number, ProductionItem>();
    for (const c of view?.cities ?? []) {
      const item = c.owner === view?.me ? pendingProduction(orders, c.id) : null;
      if (item) m.set(c.id, item);
    }
    return m;
  }, [view, orders]);

  const rangedTiles = useMemo(() => {
    if (mode !== 'ranged' || !ownSel || !view || !client) return null;
    const valid: Hex[] = [], blocked: Hex[] = [];
    for (const h of within(ownSel.pos, ownSel.range)) {
      const i = tileIndex(h, view.width, view.height);
      if (hexEq(h, ownSel.pos) || i < 0 || !view.tiles[i].visible) continue;
      const owners = [...view.cities.filter((c) => hexEq(c.pos, h)),
        ...view.units.filter((u) => hexEq(u.pos, h) && u.military)].map((x) => x.owner);
      if (!owners.length) continue;
      // Only an enemy eligible for a shot can receive a target marker.
      const attackable = owners.some((owner) => {
        if (atWar(view, owner)) return true;
        const team = view.players.find((p) => p.id === owner)?.team;
        return orders.some((o, index) => o.type === 'DeclareWar' && o.against === team
          && !useGame.getState().errors[seat].some((e) => e.index === index))
          && view.diplomacy.actions.some((a) => a.order.type === 'DeclareWar' && a.order.against === team
            && !a.blocked_reason && a.effective_turn === view.turn);
      });
      if (!attackable) continue;
      if (shotBlockReason(view, ownSel.pos, h, ownSel.kind)) { blocked.push(h); continue; }
      if (client.kind === 'remote') {
        // Online validate() only returns the last server result. Use the advertised effective
        // turn for a queued declaration, while LOS is checked locally from this PlayerView.
        valid.push(h);
      } else {
        const candidate = setUnitOrder(orders, { type: 'RangedAttack', unit: ownSel.id, target: h });
        if (!client.validate(view.me, candidate).some((e) => e.index === candidate.length - 1)) valid.push(h);
      }
    }
    return { valid, blocked };
  }, [mode, ownSel, view, client, orders, seat]);

  const territoryOverlay = useMemo((): TerritoryOverlay | null => {
    if (selection?.kind !== 'city' || !view || view.finished) return null;
    const city = view.cities.find((c) => c.id === selection.id);
    const d = city?.detail;
    if (!city || !d || city.owner !== view.me) return null;
    // Prices come straight from the selected city's view; the preview only adds what the treasury can pay.
    return { claim: territory.claimHover ? d.next_claim?.tile ?? null : null };
  }, [territory.claimHover, selection, view]);
  // A queued tile purchase keeps a mark on its tile, so the map stays clean otherwise.
  const tileBuys = useMemo(() => {
    if (!view || view.finished) return [];
    return orders.flatMap((o) => {
      if (o.type !== 'BuyTile') return [];
      const city = view.cities.find((c) => c.id === o.city && c.owner === view.me);
      const price = city?.detail?.buyable_tiles.find((b) => hexEq(b.tile, o.tile))?.price;
      return price === undefined ? [] : [{ tile: o.tile, price }];
    });
  }, [view, orders]);

  // The selected own city's improvable tiles, for the Improvements group of its production picker.
  const improve = useGame((s) => s.improve);
  const improveOverlay = useMemo((): ImproveOverlay | null => {
    if (selection?.kind !== 'city' || !view || view.finished) return null;
    const d = view.cities.find((c) => c.id === selection.id && c.owner === view.me)?.detail;
    if (!d) return null;
    const tiles = d.options.flatMap((o) => o.item.kind === 'Improvement' ? [o.item.tile] : []);
    // Tiles it is improving, and the tile a placed Yam Relay Station or Great Work will stand on (queued in the orders too).
    const queued = productionQueue(orders ?? [], selection.id, d.producing, d.production_queue ?? []);
    const building = queued.flatMap((item) => item?.kind === 'Improvement' ? [item.tile] : []);
    const sites = queued.flatMap((item) => item?.kind === 'Wonder' ? [item.tile] : []);
    if (!tiles.length && !building.length && !sites.length) return null;
    return { candidates: tiles, building, sites, hover: improve.hover, pick: improve.pick };
  }, [selection, view, improve, orders]);

  // A tile-chosen build being placed from the selected city's panel: its eligible tiles, and what the hovered one gives up.
  const place = useGame((s) => s.place);
  const placeSites = useMemo(() => {
    if (!place || selection?.kind !== 'city' || selection.id !== place.city || !view || view.finished) return null;
    const d = view.cities.find((c) => c.id === place.city && c.owner === view.me)?.detail;
    if (!d) return null;
    const faction = view.players.find((p) => p.id === view.me)?.faction;
    const own = view.cities.find((c) => c.id === place.city);
    if (place.intent === 'tile') {
      // Buying a tile: every buyable tile is lit with its price; the ones the treasury cannot pay are dimmer.
      const preview = own ? tilePurchasePreview(own, view, orders, orderErrors) : null;
      return { name: 'tile', city: own?.name ?? '', sites: (preview?.tags ?? []).map((t) => ({ tile: t.tile, lost: null, price: t.price, ring: t.ring, reason: t.reason, base: d.tile_base_price, funds: preview!.funds, queued: t.queued })) };
    }
    const sites = d.options.flatMap((o) => o.item.kind === 'Wonder' && o.item.building === place.building ? [{ tile: o.item.tile, lost: o.wonder?.lost ?? null, price: 0, ring: 0, reason: null, base: 0, funds: 0, queued: false }] : []);
    return { name: itemName({ kind: 'Building', building: place.building! }, faction), city: '', sites };
  }, [place, selection, view, orders, orderErrors]);
  const placeHover = placeSites && hover ? placeSites.sites.find((x) => hexEq(x.tile, hover.hex)) ?? null : null;
  const placeOverlay = useMemo(() => placeSites ? placeSites.sites.some((x) => x.price > 0)
    ? { candidates: placeSites.sites.filter((x) => !x.reason).map((x) => x.tile), hover: placeHover && !placeHover.reason ? placeHover.tile : null,
      prices: placeSites.sites.map((x) => ({ tile: x.tile, price: x.price, affordable: !x.reason })) }
    : { candidates: placeSites.sites.map((x) => x.tile), hover: placeHover?.tile ?? null } : null, [placeSites, placeHover]);
  // Buying a tile: an affordable lit tile says what a click does; an unaffordable one is refused in place, in red, on the tile card.
  const placeRefusal = placeSites?.name === 'tile' && hover && placeHover && placeHover.reason
    ? `Costs ${placeHover.price} Gold. You have ${placeHover.funds} Gold, ${placeHover.price - placeHover.funds} short.` : undefined;
  const placeNote = !placeSites || !hover || placeRefusal ? undefined : placeHover && placeHover.price > 0
    ? placeHover.queued ? 'Queued for this turn. Click to cancel it.'
      : `Click to buy for ${placeHover.price} Gold. ${placeSites.city}'s next tile costs ${placeHover.base} Gold${placeHover.ring > 1 ? `, and one ${placeHover.ring} tiles from the city costs ${tileRingPercent(placeHover.ring) - 100}% more` : ''}.`
    : placeSites.name === 'tile' ? 'Not a tile this city can buy. Click a lit tile, or press Esc to cancel.'
    : placeHover
    ? `${placeSites.name} here gives up ${placeHover.lost ? yieldsText(placeHover.lost) : 'this tile\'s yield'}. Click to place it here.`
    : `The ${placeSites.name} can't go here. Click a lit tile, or press Esc to cancel.`;

  // The selected own city's aircraft: their range rings, and the strike or rebase picker's candidates.
  const air = useGame((s) => s.air);
  const airOverlay = useMemo(() => {
    if (!view || !selection) return null;
    const cityId = selection.kind === 'city' ? selection.id : selection.kind === 'air' ? view.aircraft?.find((a) => a.id === selection.id)?.city : null;
    return cityId == null ? null : airOverlayFor(view, cityId, air, orders, orderErrors);
  }, [selection, view, air, orders, orderErrors]);
  // Hovering a strike candidate on the map shows the same forecast card a ground unit's shot does.
  const airShot = useMemo(() => {
    if (!view || !airOverlay || airOverlay.pickKind !== 'strike' || !hover || !airOverlay.candidates.some((c) => hexEq(c, hover.hex))) return null;
    const plane = (view.aircraft ?? []).find((a) => a.id === air.plane && a.owner === view.me);
    const city = plane && view.cities.find((c) => c.id === plane.city);
    return plane && city ? { plane, unit: aircraftAsUnit(plane, city) } : null;
  }, [view, airOverlay, hover, air.plane]);
  const airForecast: BattleForecast | null = useMemo(() => {
    if (!client || !view || !airShot || !hover) return null;
    try { return client.forecast(view.me, airShot.plane.id, hover.hex, 'Ranged'); } catch { return null; }
  }, [client, view, airShot, hover, forecastTick]);

  // Battle forecast when hovering an enemy with a military unit selected.
  const forecastTarget = activePreview?.target ?? hover?.hex;
  const blockedShot = mode === 'ranged' && ownSel && view && forecastTarget
    && distance(ownSel.pos, forecastTarget) <= ownSel.range
    ? shotBlockReason(view, ownSel.pos, forecastTarget, ownSel.kind) : null;
  const forecast: BattleForecast | null = useMemo(() => {
    if (!client || !view || !ownSel || !ownSel.military || !forecastTarget) return null;
    if (!enemyAt(view, forecastTarget)) return null;
    if (mode === 'ranged' && (distance(ownSel.pos, forecastTarget) > ownSel.range || blockedShot)) return null;
    try {
      return client.forecast(view.me, ownSel.id, forecastTarget, mode === 'ranged' ? 'Ranged' : 'Melee');
    } catch {
      return null;
    }
  }, [client, view, ownSel, forecastTarget, mode, forecastTick, blockedShot]);

  // The whole fight at an enemy city: core pools this attack with every other attack of yours queued on the same tile this turn.
  const assault: CityAssaultForecast | null = useMemo(() => {
    if (!client?.assault || !view || !ownSel || !forecast || !forecastTarget) return null;
    if (!view.cities.some((c) => hexEq(c.pos, forecastTarget) && atWar(view, c.owner))) return null;
    const alongside: AlongsideAttack[] = (orders ?? []).flatMap((o): AlongsideAttack[] => {
      if (o.type === 'RangedAttack') return o.unit !== ownSel.id && hexEq(o.target, forecastTarget) ? [{ unit: o.unit, mode: 'Ranged' }] : [];
      if (o.type === 'MoveUnit' || o.type === 'MoveUnitPath') return o.unit !== ownSel.id && hexEq(o.to, forecastTarget) ? [{ unit: o.unit, mode: 'Melee' }] : [];
      return [];
    });
    try {
      return client.assault(view.me, ownSel.id, forecastTarget, forecast.ranged ? 'Ranged' : 'Melee', alongside);
    } catch {
      return null;
    }
  }, [client, view, ownSel, forecast, forecastTarget, orders, forecastTick]);

  // A Keshig (a mounted archer) shoots or charges. Out of ranged mode, a target it could do both to shows both
  // forecasts on one card; when the shot is not on offer the card says why, next to the charge.
  const bow = !!ownSel && ownSel.military && ownSel.ranged > 0 && attacksInMelee(ownSel.kind);
  const shotWhy: string | null = (() => {
    if (!bow || mode === 'ranged' || !ownSel || !view || !forecastTarget || !forecast || forecast.ranged) return null;
    if (distance(ownSel.pos, forecastTarget) > ownSel.range) {
      return `No shot from here: this unit shoots ${ownSel.range === 1 ? 'only at a target next to it' : `up to ${ownSel.range} tiles`}, and the target is ${distance(ownSel.pos, forecastTarget)} tiles away. Right-click to charge it.`;
    }
    if (!rangedEnemyAt(view, forecastTarget)) return 'A shot hits only military units and cities. Right-click to ride onto it.';
    return shotBlockReason(view, ownSel.pos, forecastTarget, ownSel.kind);
  })();
  const shotForecast: BattleForecast | null = useMemo(() => {
    if (!bow || mode === 'ranged' || shotWhy || !forecast || forecast.ranged || !client || !view || !ownSel || !forecastTarget) return null;
    try {
      return client.forecast(view.me, ownSel.id, forecastTarget, 'Ranged');
    } catch {
      return null;
    }
  }, [client, view, ownSel, forecastTarget, mode, forecastTick, bow, shotWhy, forecast]);

  // Everything the draw loop reads, kept in a ref so the loop never re-subscribes.
  const frame = useRef({
    anchoringPreview: !!activePreview,
    view,
    selUnit,
    selection,
    pathInfo,
    pathAttack,
    pendingLines,
    pendingMarks,
    cityFocuses,
    cityBuilds,
    rangedTiles,
    territoryOverlay,
    tileBuys,
    improveOverlay,
    placeOverlay,
    airOverlay,
    goalPreview,
    hover,
    hoverSelect: false,
    lens,
    lensTiles,
    overlays,
    yieldTiles: overlayTiles.yields,
    borderTiles: overlayTiles.borders,
    cues: null as { list: MapCue[]; start: number } | null,
    arrow: null as { from: Hex | null; to: Hex; start: number } | null,
    anim,
    seat,
    reducedMotion,
    combatFx: { enabled: combatFx.enabled },
    celebration,
    scenarioTargets,
    siege,
  });
  const nextFrame = { anchoringPreview: !!activePreview, lens, lensTiles, overlays, yieldTiles: overlayTiles.yields, borderTiles: overlayTiles.borders, view, selUnit, selection, pathInfo, pathAttack, pendingLines, pendingMarks, cityFocuses, cityBuilds, rangedTiles, goalPreview, territoryOverlay, tileBuys, improveOverlay, placeOverlay, airOverlay, hover, hoverSelect: intent === 'select', cues, arrow, anim, seat, reducedMotion, combatFx: { enabled: combatFx.enabled }, celebration, scenarioTargets, siege };
  // Redraw only when something the map draws changed: a render for a card or a panel leaves the canvas alone.
  // A change of the tile under the pointer alone repaints only the hover mark and the route (hoverClipBoxes).
  if (hoverOnlyChange(frame.current, nextFrame)) hoverDirty.current = true;
  else if (frameChanged(frame.current, nextFrame)) dirty.current = true;
  frame.current = nextFrame;

  // The panels that can cover a tile: a tile the player is sent to must be left clear of them.
  const hudObstacles = (canvas: HTMLCanvasElement, game: Element) => {
    const r = canvas.getBoundingClientRect();
    return [...game.querySelectorAll('.moment-lane, .report, .vp, .topbar, .left-dock > *, .seatbar, .minimap, .map-lenses')]
      .map((el) => el.getBoundingClientRect()).filter((b) => b.width && b.height)
      .map((b) => ({ left: b.left - r.left - 10, right: b.right - r.left + 10, top: b.top - r.top - 10, bottom: b.bottom - r.top + 10 }));
  };

  // Center on focus requests; a `clear` request (a turn report line) settles on the nearest tile position no panel covers.
  useEffect(() => {
    if (!focus) return;
    let p = hexToPixel(focus.hex, HEX_SIZE);
    const canvas = canvasRef.current, game = canvas?.closest('.game');
    if ((focus.clear || selection) && canvas && game) {
      const r = canvas.getBoundingClientRect(), zoom = cam.current.zoom;
      if (r.width && r.height) {
        const origin = { x: r.width / 2, y: r.height / 2 };
        const visible = visibleTilePoint(origin, Math.max(selection ? 100 : 48, HEX_SIZE * zoom + 24),
          { left: 8, top: 8, right: r.width - 8, bottom: r.height - 8 }, hudObstacles(canvas, game));
        if (visible) p = { x: p.x + (origin.x - visible.x) / zoom, y: p.y + (origin.y - visible.y) / zoom };
      }
    }
    const start = { ...cam.current };
    const t0 = performance.now();
    const step = (t: number) => {
      const k = Math.min(1, (t - t0) / 380);
      const e = 1 - Math.pow(1 - k, 3);
      cam.current.x = start.x + (p.x - start.x) * e;
      cam.current.y = start.y + (p.y - start.y) * e;
      dirty.current = true;
      if (k < 1) focusAnimation.current = requestAnimationFrame(step);
    };
    // First focus of a game snaps instead of flying across the planet.
    if (still || focus.seq <= 1 || (start.x === 0 && start.y === 0)) {
      cam.current.x = p.x;
      cam.current.y = p.y;
      dirty.current = true;
    } else focusAnimation.current = requestAnimationFrame(step);
    return () => cancelAnimationFrame(focusAnimation.current);
  }, [focus, still]);

  // A click can open the left panel over its own unit. Once selection/focus motion settles,
  // reveal the unit and its adjacent destinations; resize follows the same measured HUD bounds.
  useEffect(() => {
    if (phone || !selection || !view) return;
    const at = selection.kind === 'unit' ? view.units.find((u) => u.id === selection.id)?.pos
      : view.cities.find((c) => c.id === (selection.kind === 'air'
        ? view.aircraft?.find((p) => p.id === selection.id)?.city : selection.id))?.pos;
    const canvas = canvasRef.current, game = canvas?.closest('.game');
    if (!at || !canvas || !game) return;
    let timer: ReturnType<typeof setTimeout>;
    const reveal = () => {
      const r = canvas.getBoundingClientRect(), camera = cam.current;
      if (!r.width || !r.height) return;
      const world = hexToPixel(at, HEX_SIZE);
      const origin = { x: (world.x - camera.x) * camera.zoom + r.width / 2, y: (world.y - camera.y) * camera.zoom + r.height / 2 };
      const bounds = { left: 8, top: 8, right: r.width - 8, bottom: r.height - 8 };
      // A unit already off screen is one the player panned or jumped away from: a panel opening is no reason to fly back.
      if (origin.x < 0 || origin.y < 0 || origin.x > r.width || origin.y > r.height) return;
      const obstacles = hudObstacles(canvas, game);
      const visible = visibleTilePoint(origin, Math.max(100, HEX_SIZE * camera.zoom * 2.5), bounds, obstacles)
        ?? visibleTilePoint(origin, HEX_SIZE * camera.zoom + 12, bounds, obstacles);
      if (!visible) return;
      camera.x += (origin.x - visible.x) / camera.zoom;
      camera.y += (origin.y - visible.y) / camera.zoom;
      dirty.current = true;
    };
    const schedule = () => { clearTimeout(timer); timer = setTimeout(reveal, 420); };
    const observer = new ResizeObserver(schedule);
    observer.observe(canvas);
    for (const el of game.querySelectorAll('.left-dock, .unit-panel, .map-lenses')) observer.observe(el);
    // Explicit map navigation wins over an outstanding layout nudge.
    const cancel = () => clearTimeout(timer);
    // A player already moving the mouse to the next click keeps the map still under the cursor.
    let aimFrom: Point | null = null;
    const aim = (e: PointerEvent) => {
      aimFrom ??= { x: e.clientX, y: e.clientY };
      if (Math.hypot(e.clientX - aimFrom.x, e.clientY - aimFrom.y) > 8) cancel();
    };
    canvas.addEventListener('pointermove', aim);
    canvas.addEventListener('pointerdown', cancel);
    canvas.addEventListener('wheel', cancel);
    canvas.addEventListener('keydown', cancel);
    schedule();
    return () => {
      clearTimeout(timer); observer.disconnect();
      canvas.removeEventListener('pointermove', aim);
      canvas.removeEventListener('pointerdown', cancel);
      canvas.removeEventListener('wheel', cancel);
      canvas.removeEventListener('keydown', cancel);
    };
  }, [selection?.kind, selection?.id, phone]);

  // Legendary moments keep their tile clear of the banner/report and live controls,
  // including after fonts or viewport sizes change.
  useEffect(() => {
    const moment = celebration.active;
    if (moment?.tier !== 'legendary' || !moment.at) return;
    const world = hexToPixel(moment.at, HEX_SIZE);
    let measureFrame = 0;
    const reveal = () => {
      const canvas = canvasRef.current, game = canvas?.closest('.game');
      if (!canvas || !game) return;
      const r = canvas.getBoundingClientRect();
      if (!r.width || !r.height) return;
      cancelAnimationFrame(focusAnimation.current);
      const start = { ...cam.current };
      const origin = { x: (world.x - start.x) * start.zoom + r.width / 2, y: (world.y - start.y) * start.zoom + r.height / 2 };
      const visible = visibleTilePoint(origin, Math.max(48, HEX_SIZE * start.zoom + 24),
        { left: 8, top: 8, right: r.width - 8, bottom: r.height - 8 }, hudObstacles(canvas, game));
      if (!visible) return;
      const target = { x: start.x + (origin.x - visible.x) / start.zoom, y: start.y + (origin.y - visible.y) / start.zoom };
      if (Math.abs(target.x - start.x) + Math.abs(target.y - start.y) < .5) return;
      const t0 = performance.now();
      const step = (t: number) => {
        const k = still ? 1 : Math.min(1, (t - t0) / 380);
        const ease = 1 - (1 - k) ** 3;
        cam.current.x = start.x + (target.x - start.x) * ease;
        cam.current.y = start.y + (target.y - start.y) * ease;
        dirty.current = true;
        if (k < 1) focusAnimation.current = requestAnimationFrame(step);
      };
      step(t0);
    };
    const schedule = () => { cancelAnimationFrame(measureFrame); measureFrame = requestAnimationFrame(reveal); };
    const observer = new ResizeObserver(schedule);
    const game = canvasRef.current?.closest('.game');
    for (const el of game?.querySelectorAll('.map-canvas, .moment-lane, .report') ?? []) observer.observe(el);
    schedule();
    return () => { observer.disconnect(); cancelAnimationFrame(measureFrame); cancelAnimationFrame(focusAnimation.current); };
  }, [celebration.active, celebration.holding, focus, still]);

  // Resize.
  useEffect(() => {
    const wrap = wrapRef.current!;
    const canvas = canvasRef.current!;
    let resizeFrame = 0;
    const resize = () => {
      resizeFrame = 0;
      const r = wrap.getBoundingClientRect();
      if (!r.width || !r.height) return;
      const flight = r;
      flightSize.current = { w: flight.width, h: flight.height, x: 0, y: 0 };
      const dpr = Math.min(2, window.devicePixelRatio || 1);
      size.current = { w: r.width, h: r.height, dpr };
      for (const layer of [canvas, flightRef.current!]) {
        const bounds = layer === canvas ? r : flight;
        const width = Math.round(bounds.width * dpr), height = Math.round(bounds.height * dpr);
        if (layer.width !== width) layer.width = width;
        if (layer.height !== height) layer.height = height;
        if (layer.style.width !== `${bounds.width}px`) layer.style.width = `${bounds.width}px`;
        if (layer.style.height !== `${bounds.height}px`) layer.style.height = `${bounds.height}px`;
      }
      dirty.current = true;
    };
    // Canvas dimensions notify descendant observers. Write them on the next frame,
    // never while the sheet/lens/map ResizeObservers are delivering notifications.
    const ro = new ResizeObserver(() => { if (!resizeFrame) resizeFrame = requestAnimationFrame(resize); });
    resize();
    ro.observe(wrap);
    onIconLoad(() => (dirty.current = true));
    onMapFeatureLoad(() => (dirty.current = true));
    const offCityArt = onCityArtLoad(() => (dirty.current = true));
    const offImprovementArt = onImprovementArtLoad(() => (dirty.current = true));
    const offLandmarkArt = onLandmarkArtLoad(() => (dirty.current = true));
    const offWonderArt = onWonderArtLoad(() => (dirty.current = true));
    void document.fonts?.ready.then(() => (dirty.current = true));
    // Debug/testing hook: where a hex is on screen (CSS pixels, relative to the canvas).
    (window as unknown as { legatumMap: unknown }).legatumMap = {
      toScreen: (h: Hex) => {
        const p = hexToPixel(h, HEX_SIZE);
        const c = cam.current;
        return { x: (p.x - c.x) * c.zoom + size.current.w / 2, y: (p.y - c.y) * c.zoom + size.current.h / 2 };
      },
      /** Frame timing: { frames, drawMs, maxMs, workMs } since the last reset(). */
      perf,
      /** Terrain chunk bookkeeping after the last draw: cached, renders so far, visible chunks missing or stale. */
      get terrain() { return terrainStats(); },
      /** The report arrow as last drawn (attacker → target, with its fade), or null. */
      get reportArrow() { return drawnArrow.current; },
      /** The lens tiles as last built (Settle: one per legal site, with its band). */
      get lensTiles() { return frame.current.lensTiles; },
      /** The city banners as last drawn: the name plate (hover) and the whole cover with meters (click), in CSS pixels. */
      get banners() { return bannerHits.current.map((b) => ({ city: b.city.id, rect: b.rect, cover: b.cover, parts: b.parts, far: b.far })); },
      /** The cities drawn alarmed (under attack), by id. */
      get siege() { return [...frame.current.siege.keys()]; },
      /** The placement overlay as last built: the lit tiles of a tile-chosen build, or null. */
      get place() { return frame.current.placeOverlay; },
      /** The improvement overlay as last built: improvable tiles, and tiles being improved or holding a placed Wonder site. */
      get improve() { return frame.current.improveOverlay; },
      /** The aircraft overlay as last built: ring radii and labels, the open picker's candidates. */
      get air() { const a = frame.current.airOverlay; return a ? { center: a.center, rings: a.rings.map((r) => [r.radius, r.label]), pick: a.pickKind, candidates: a.candidates } : null; },
      camera: cam.current,
      redraw: () => (dirty.current = true),
      /** Dev switch for the territory border look (`current`, `a`, `b`, `c`); also `?borders=` in the URL. */
      setBorderVariant: (v: BorderVariant) => { setBorderVariant(v); dirty.current = true; },
    };
    // A look change swaps the map palette: redraw.
    const unregister = registerMapCamera({
      viewport: () => ({ ...cam.current, w: size.current.w, h: size.current.h }),
      center: (x, y) => {
        cancelAnimationFrame(focusAnimation.current);
        cam.current.x = x;
        cam.current.y = y;
        dirty.current = true;
      },
    });
    return () => {
      unregister();
      offCityArt();
      offImprovementArt();
      offLandmarkArt();
      offWonderArt();
      ro.disconnect();
      cancelAnimationFrame(resizeFrame);
    };
  }, []);

  // Compass: keep clear of panels and the labels actually drawn in the current frame.
  // Panel rects are sampled a few times a second rather than read every frame.
  const placeCompass = () => {
    const el = compassRef.current;
    const v = frame.current.view;
    if (!el || !v) return;
    const { x, y, covered } = compassPlace(worldBounds(v), cam.current, size.current.w, size.current.h, [...panels.current, ...labels.current]);
    el.style.transform = `translate(${Math.round(x)}px, ${Math.round(y)}px)`;
    el.classList.toggle('covered', covered);
  };
  useEffect(() => {
    const sample = () => {
      const wrap = wrapRef.current;
      if (!wrap) return;
      const o = wrap.getBoundingClientRect();
      panels.current = [...document.querySelectorAll('.game .panel, .game .vp, .game .report, .map-lenses, .first-game-hint, .topbar')].map((e) => {
        const r = e.getBoundingClientRect();
        return { x: r.left - o.left, y: r.top - o.top, w: r.width, h: r.height };
      });
      placeCompass();
    };
    sample();
    const t = window.setInterval(sample, 250);
    return () => window.clearInterval(t);
  }, []); // placeCompass reads refs only

  // Draw loop.
  useEffect(() => {
    const canvas = canvasRef.current!;
    const ctx = canvas.getContext('2d')!;
    let raf = 0;
    let lastPulse = 0;
    let lastSmoulder = 0;
    let lastFonts = -1;
    let reported = '';
    let lastCam = { x: NaN, y: NaN, zoom: NaN };
    let heldMoments = false;
    let lastIdle = -Infinity;
    // The selection ring as the last frame drew it, and the clock of the last frame that drew more than the ring.
    let lastRing: SelectionRing | null = null;
    // The smoke and siege-ring boxes the last full frame drew: what an ambient-only frame repaints.
    let lastAmbient: Rect[] = [];
    // Idle life's clock, and where it painted last (an idle tick repaints those boxes and the new ones).
    let lastIdleT = 0;
    let lastIdleBoxes: Rect[] = [];
    // Where the last frame drew the hover mark and the route preview.
    let lastHoverBoxes: Rect[] = [];
    let lastFullT = 0;
    let flightDrawn = false;
    // Whether the last drawn frame had anything for idle life to move (smoke, water) in view.
    let idleInView = true;
    const loop = (t: number) => {
      raf = requestAnimationFrame(loop);
      const f = frame.current;
      if (!f.view) return;
      // Combat playback (combatFx.ts) replaces the plain glide while it runs: it moves units round by round after the volley.
      const still = motionStill(f.reducedMotion, f.combatFx.enabled);
      const plan = !still ? f.anim?.plan ?? null : null;
      const fxT = plan && f.anim ? (t - f.anim.start) : -1;
      const fxRunning = !!plan && fxLive(plan, fxT);
      // Map animations Off is the reduced-motion map: units step straight to where they ended.
      // Payoff moments (momentFx.ts) on the same speed-scaled clock as the combat playback.
      const mplan = !still ? f.anim?.moments ?? null : null;
      const mT = mplan && f.anim ? t - f.anim.start : -1;
      const momentsRunning = !!mplan && momentsLive(mplan, mT);
      // One more frame after the last moment, so the map lets go of everything it held back.
      const momentsEnding = !momentsRunning && heldMoments;
      heldMoments = momentsRunning;
      const animating = fxRunning || momentsRunning || momentsEnding || (!plan && !still && !!f.anim && f.anim.paths.size > 0 && t - f.anim.start < f.anim.duration + 50);
      const pulse = !still && !!f.selection && t - lastPulse > 33;
      const cueDelay = plan ? (hex: Hex) => plan.cueDelay.get(`${hex.q},${hex.r}`) ?? 0 : undefined;
      const cueSpan = CUE_HOLD_MS + CUE_FADE_MS + 100 + (plan ? Math.max(0, ...plan.cueDelay.values()) : 0);
      const cueLive = !!f.cues && t - f.cues.start < cueSpan;
      const arrowLive = !!f.arrow && t - f.arrow.start < ARROW_HOLD_MS + ARROW_FADE_MS + 100;
      // A captured city's celebration waits for the capture in the playback (the taker's unit arriving), not before it.
      // A founded city's or a finished building's celebration waits for its moment on the map too.
      const at = f.celebration.active?.at;
      const capT = at && f.anim ? (plan ? captureTime(plan, at) : null) ?? (mplan ? momentTime(mplan, at) : null) : null;
      const celebStarted = capT != null && f.anim ? Math.max(f.celebration.started, f.anim.start + capT) : f.celebration.started;
      const celebrating = !still && !!f.celebration.active && t - celebStarted < EFFECT_MS + 50;
      const siegePulse = !still && f.siege.size > 0 && t - lastPulse > 33;
      // Idle life (idleLife.ts) redraws the map a few times a second, never every frame, with map animations on.
      const idleLife = !still;
      const idleTick = idleLife && idleInView && t - lastIdle > 1000 / IDLE_FPS;
      if (idleTick) lastIdle = t;
      // Damaged cities smoulder while Map animations are on (about 20 frames a second is enough for smoke).
      const smoulder = !still && hasVisibleSmoulder(f.view.cities, cam.current, size.current.w, size.current.h);
      const smoulderTick = smoulder && t - lastSmoulder > 50;
      const hoverMoved = hoverDirty.current && !dirty.current;
      if (!dirty.current && !hoverMoved && !animating && !pulse && !cueLive && !arrowLive && !celebrating && !siegePulse && !smoulderTick && !idleTick) return;
      if (pulse || siegePulse) lastPulse = t;
      // Nothing but the ambient motion wants a frame (the selection pulse, a siege ring breathing, a damaged city's
      // smoke, idle life): repaint just their boxes, with the rest of the map's clock held. A full redraw of a crowded
      // late-game map 20 to 30 times a second only for these kept the main thread busy and made hover and clicks lag.
      // The pointer reaching a new tile is the same kind of frame: the old and new hover marks and routes are a few boxes.
      const pulseOnly = (pulse || siegePulse || smoulderTick || idleTick || hoverMoved) && !dirty.current && !animating && !cueLive && !arrowLive && !celebrating
        && !f.scenarioTargets?.length && (!pulse || !!lastRing) && (!(siegePulse || smoulderTick) || lastAmbient.length > 0);
      const drawT = pulseOnly ? lastFullT : t;
      if (!pulseOnly) lastFullT = t;
      // Idle life keeps its own clock: it moves on a full frame or an idle tick, and holds through the other clipped frames.
      const idleT = !pulseOnly || idleTick ? t : lastIdleT;
      lastIdleT = idleT;
      const idleNow = !idleLife ? [] : idleTick || !pulseOnly ? idleClipBoxes(f.view, cam.current, size.current.w, size.current.h, idleT) : lastIdleBoxes;
      const hoverNow = hoverClipBoxes({ view: f.view, cam: cam.current, width: size.current.w, height: size.current.h, hover: f.hover?.hex ?? null,
        path: f.pathInfo?.steps ?? null, pathRisk: f.pathInfo?.risk?.hex ?? null, pathUnreachable: f.pathInfo?.unreachable ?? null,
        selectedUnit: f.selection?.kind === 'unit' ? f.selection.id : null, pendingMarks: f.pendingMarks });
      const clipBoxes = pulseOnly ? ambientClipRects(lastRing, [...lastAmbient, ...(idleTick ? [...lastIdleBoxes, ...idleNow] : []), ...(hoverMoved ? [...lastHoverBoxes, ...hoverNow] : [])], size.current.dpr) : [];
      lastIdleBoxes = idleNow;
      lastHoverBoxes = hoverNow;
      hoverDirty.current = false;
      if (smoulder) lastSmoulder = t;
      dirty.current = false;
      clampCamera(cam.current, f.view, size.current.w, size.current.h);
      // Tell the minimap where the map looks, only when that changed.
      const where = `${cam.current.x},${cam.current.y},${cam.current.zoom},${size.current.w},${size.current.h}`;
      if (where !== reported) { reported = where; notifyMapCamera(); }
      let unitPos = new Map<number, { x: number; y: number }>();
      let unitShift: Map<number, { x: number; y: number }> | undefined;
      let ghosts: FxGhost[] | undefined;
      // Marching (momentFx.ts): a stride in the step and dust off the heels, with map animations on.
      let march: Map<number, March> | undefined;
      const marching = !still;
      if (plan && fxRunning) {
        const u = fxUnits(plan, fxT);
        unitPos = u.pos; unitShift = u.shift; ghosts = u.ghosts;
        if (marching) {
          march = new Map();
          for (const [id, steps] of plan.moves) {
            const m = marchOf((tt) => stepPos(steps, tt), fxT, steps.findIndex((s) => fxT >= s.t0 && fxT <= s.t1) >= 0);
            if (m) march.set(id, m);
          }
        }
      } else if (!plan && f.anim && animating) {
        const k = Math.min(1, Math.max(0, (t - f.anim.start) / f.anim.duration));
        const e = k < 0.5 ? 2 * k * k : 1 - Math.pow(-2 * k + 2, 2) / 2;
        for (const [id, path] of f.anim.paths) {
          if (path.length < 2) continue;
          const pos = e * (path.length - 1);
          const i = Math.min(path.length - 2, Math.floor(pos));
          const fr = pos - i;
          const a = hexToPixel(path[i], HEX_SIZE);
          const b = hexToPixel(path[i + 1], HEX_SIZE);
          unitPos.set(id, { x: a.x + (b.x - a.x) * fr, y: a.y + (b.y - a.y) * fr });
        }
        if (marching && k > 0 && k < 1) {
          const anim = f.anim;
          march = new Map();
          for (const [id, path] of anim.paths) {
            if (path.length < 2) continue;
            const at = (tt: number) => {
              const kk = Math.min(1, Math.max(0, (tt - anim.start) / anim.duration));
              const ee = kk < 0.5 ? 2 * kk * kk : 1 - Math.pow(-2 * kk + 2, 2) / 2;
              const pos = ee * (path.length - 1);
              const i = Math.min(path.length - 2, Math.floor(pos));
              const a = hexToPixel(path[i], HEX_SIZE), b = hexToPixel(path[i + 1], HEX_SIZE);
              return { x: a.x + (b.x - a.x) * (pos - i), y: a.y + (b.y - a.y) * (pos - i) };
            };
            const m = marchOf(at, t, true);
            if (m) march.set(id, m);
          }
        }
      }
      const gift = f.celebration.active;
      if (!still && gift?.at && gift.scout) {
        const k = Math.min(1, Math.max(0, (t - celebStarted) / FLIGHT_MS));
        const e = 1 - Math.pow(1 - k, 3);
        const a = hexToPixel(gift.at, HEX_SIZE), b = hexToPixel(gift.scout.to, HEX_SIZE);
        unitPos.set(gift.scout.unit, { x: a.x + (b.x - a.x) * e, y: a.y + (b.y - a.y) * e });
      }
      if (pulseOnly) {
        perf.partialFrames++;
        ctx.save();
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.beginPath();
        for (const box of clipBoxes) ctx.rect(box.x, box.y, box.w, box.h);
        ctx.clip();
      }
      const { pending, labelRects, bannerRects, bannerHits: hits, liftedHits: lifted, idle: idleSeen, ring, ambient } = draw(ctx, {
        view: f.view,
        cam: cam.current,
        width: size.current.w,
        height: size.current.h,
        dpr: size.current.dpr,
        time: still ? 0 : drawT,
        pulseTime: still ? 0 : t,
        hover: f.hover?.hex ?? null,
        hoverSelect: f.hoverSelect,
        lens: f.lens,
        lensTiles: f.lensTiles,
        overlays: f.overlays,
        yieldTiles: f.yieldTiles,
        borderTiles: f.borderTiles,
        lensAvoid: panels.current,
        selectedUnit: f.selection?.kind === 'unit' ? f.selection.id : null,
        selectedAir: f.selection?.kind === 'air' ? f.selection.id : null,
        selectedCity: f.selection?.kind === 'city' ? f.selection.id : null,
        path: f.pathInfo?.steps ?? null,
        pathAttack: f.pathAttack,
        pathUnreachable: f.pathInfo?.unreachable ?? null,
        pathRisk: f.pathInfo?.risk?.hex ?? null,
        pendingLines: f.pendingLines,
        pendingMarks: f.pendingMarks,
        cityFocuses: f.cityFocuses,
        cityBuilds: f.cityBuilds,
        cues: f.cues ? cuesAt(f.cues.list, t - f.cues.start, still, cueDelay) : [],
        reportArrow: (drawnArrow.current = f.arrow ? arrowAt(f.arrow, t - f.arrow.start) : null),
        celebration: !still && f.celebration.active && t >= celebStarted ? { moment: f.celebration.active, elapsed: t - celebStarted, reduced: still, targets: celebrationTargets.current } : undefined,
        rangedTiles: f.rangedTiles,
        goalPreview: f.goalPreview,
        territory: f.territoryOverlay,
        tileBuys: f.tileBuys,
        improve: f.improveOverlay,
        place: f.placeOverlay,
        air: f.airOverlay,
        unitPos,
        unitShift,
        ghosts,
        march,
        idle: idleLife,
        fx: plan && fxRunning ? { plan, t: fxT } : null,
        moments: mplan && momentsRunning ? { plan: mplan, t: mT, hold: holdAt(mplan, mT) } : null,
        ambientT: still ? null : t,
        idleT,
        seatColor: mapPlayerColor(f.view?.me),
        scenarioTargets: f.scenarioTargets,
        siege: f.siege,
        reducedMotion: still,
      });
      if (pulseOnly) ctx.restore();
      lastRing = ring ?? null;
      // A new route is a new box for the pulse frames to repaint.
      if (!pulseOnly || hoverMoved) lastAmbient = ambient;
      const flightCtx = flightRef.current?.getContext('2d');
      const flightLive = !!f.celebration.active?.at && t >= celebStarted;
      if (flightCtx && (flightLive || flightDrawn)) {
        flightDrawn = flightLive;
        const { w, h, dpr } = size.current;
        const flight = flightSize.current;
        flightCtx.setTransform(dpr, 0, 0, dpr, 0, 0); flightCtx.clearRect(0, 0, flight.w, flight.h);
        if (f.celebration.active?.at && t >= celebStarted) {
          const from = hexToPixel(f.celebration.active.at, HEX_SIZE);
          const origin = { x: (from.x - cam.current.x) * cam.current.zoom + w / 2 + flight.x, y: (from.y - cam.current.y) * cam.current.zoom + h / 2 + flight.y };
          const bounds = canvas.getBoundingClientRect(), race = raceRect.current;
          const momentCard = document.querySelector('.moment-card:not(.unlock)')?.getBoundingClientRect();
          const targets = Object.fromEntries(Object.entries(celebrationTargets.current).map(([key, point]) => [key, { x: point.x + flight.x, y: point.y + flight.y }]));
          drawCounterFlights(flightCtx, { moment: f.celebration.active, elapsed: t - celebStarted,
            reduced: still, targets }, origin, flight.w, flight.h,
            race ? { x: race.x - bounds.x + flight.x, y: race.y - bounds.y + flight.y, width: race.width, height: race.height } : null,
            momentCard ? { x: momentCard.x - bounds.x + flight.x, y: momentCard.y - bounds.y + flight.y, width: momentCard.width, height: momentCard.height } : null,
            '#b68c25', '#fff6da', 'sans-serif');
        }
      }
      labels.current = labelRects;
      idleInView = idleSeen;
      banners.current = bannerRects ?? [];
      bannerHits.current = hits ?? [];
      liftedHits.current = lifted ?? [];
      placeCompass();
      const c = cam.current;
      if (c.x !== lastCam.x || c.y !== lastCam.y || c.zoom !== lastCam.zoom) {
        lastCam = { x: c.x, y: c.y, zoom: c.zoom };
        placeInspector.current?.();
      }
      // Terrain chunks still rendering, or fonts and icons arriving: keep drawing.
      if (pending || fontEpoch() !== lastFonts) dirty.current = true;
      lastFonts = fontEpoch();
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, []);

  // Keyboard panning.
  useEffect(() => {
    const keys = new Set<string>();
    let raf = 0;
    const stop = () => {
      keys.clear();
      cancelAnimationFrame(raf);
    };
    const tick = () => {
      // Focus or an overlay may change while a pan key is still held down.
      if (!gameplayFocusAllowed(document.activeElement, 'arrowup')) { stop(); return; }
      const sp = 14 / cam.current.zoom;
      if (keys.has('w') || keys.has('arrowup')) cam.current.y -= sp;
      if (keys.has('s') || keys.has('arrowdown')) cam.current.y += sp;
      if (keys.has('a') || keys.has('arrowleft')) cam.current.x -= sp;
      if (keys.has('d') || keys.has('arrowright')) cam.current.x += sp;
      dirty.current = true;
      if (keys.size) raf = requestAnimationFrame(tick);
    };
    const panKeys = ['w', 'a', 's', 'd', 'arrowup', 'arrowdown', 'arrowleft', 'arrowright'];
    const down = (e: KeyboardEvent) => {
      if (!gameplayKeyAllowed(e)) return;
      const k = e.key.toLowerCase();
      if (!panKeys.includes(k)) return;
      e.preventDefault();
      if (!keys.size) raf = requestAnimationFrame(tick);
      keys.add(k);
    };
    const up = (e: KeyboardEvent) => {
      keys.delete(e.key.toLowerCase());
      if (!keys.size) stop();
    };
    const focusChanged = () => { if (!gameplayFocusAllowed(document.activeElement, 'arrowup')) stop(); };
    window.addEventListener('keydown', down);
    window.addEventListener('keyup', up);
    window.addEventListener('blur', stop);
    window.addEventListener('focusin', focusChanged);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener('keydown', down);
      window.removeEventListener('keyup', up);
      window.removeEventListener('blur', stop);
      window.removeEventListener('focusin', focusChanged);
    };
  }, []);

  // Delay the tile info card so it doesn't flicker while sweeping the mouse.
  useEffect(() => {
    setTileHoverReady(false);
    if (!hover) return;
    const t = setTimeout(() => setTileHoverReady(true), 650);
    return () => clearTimeout(t);
  }, [hover?.hex.q, hover?.hex.r]); // eslint-disable-line react-hooks/exhaustive-deps

  const hexAtScreen = (sx: number, sy: number): Hex => {
    const w = screenToWorld(cam.current, size.current.w, size.current.h, sx, sy);
    return pixelToHex(w.x, w.y, HEX_SIZE);
  };
  // What a click or hover at a screen point acts on: a city banner is drawn above the units, so the
  // banner's city wins where the banner overhangs the hex above, then a token raised clear of a banner
  // (its head pokes above its own hex), then the hex (except while picking a tile or a plane's target).
  const targetAtScreen = (sx: number, sy: number): Hex => {
    const s = useGame.getState();
    if (s.place || (s.air.plane !== null && s.air.what)) return hexAtScreen(sx, sy);
    return bannerCityAt(bannerHits.current, sx, sy)?.pos
      ?? liftedHits.current.findLast(({ rect: r }) => sx >= r.x && sx <= r.x + r.w && sy >= r.y && sy <= r.y + r.h)?.hex
      ?? hexAtScreen(sx, sy);
  };

  const local = (e: { clientX: number; clientY: number }) => {
    const r = canvasRef.current!.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  const zoomAt = (sx: number, sy: number, factor: number) => {
    Object.assign(cam.current, zoomCamera(cam.current, size.current, { x: sx, y: sy }, { x: sx, y: sy }, factor));
    dirty.current = true;
  };

  const clickHex = (h: Hex, allowRanged = true) => {
    const s = useGame.getState();
    const v = s.views?.[s.seat];
    if (!v || tileIndex(h, v.width, v.height) < 0) return;
    if (allowRanged && s.mode === 'ranged' && !v.finished && !isEliminated(v)) {
      rangedSelected(h);
      return;
    }
    if (allowRanged && s.mode === 'move' && !v.finished && !isEliminated(v)) {
      moveSelected(h);
      return;
    }
    // Placing a tile-chosen build: a lit tile is the pick (the city panel turns it into the order);
    // any other click cancels the placement and acts as usual. Buying a tile queues its order here.
    if (s.place) {
      const own = s.selection?.kind === 'city' && s.selection.id === s.place.city ? v.cities.find((c) => c.id === s.place!.city && c.owner === v.me) : undefined;
      if (s.place.intent === 'tile') {
        const tag = own ? tilePurchasePreview(own, v, s.orders[s.seat], s.errors[s.seat]).tags.find((t) => hexEq(t.tile, h)) : undefined;
        if (!v.finished && own && tag?.affordable) {
          s.set({ place: null });
          s.editOrders((ord) => toggleBuyTile(ord, own.id, h), 'purchase');
          feel.select();
          return;
        }
        // A lit tile the treasury cannot pay is refused in place: the mode stays open and the tile card (still under the cursor) says why.
        if (own && tag) return;
      } else if (!v.finished && own?.detail?.options.some((o) => o.item.kind === 'Wonder' && o.item.building === s.place!.building && hexEq(o.item.tile, h))) {
        s.set({ place: { ...s.place, pick: h } });
        feel.select();
        return;
      }
      s.set({ place: null });
    }
    // The aircraft picker is open on the selected city: a click on one of its ringed candidates orders it.
    if ((s.selection?.kind === 'city' || s.selection?.kind === 'air') && s.air.plane !== null && s.air.what) {
      const city = v.cities.find((c) => c.id === (s.selection!.kind === 'city' ? s.selection!.id : v.aircraft?.find((a) => a.id === s.air.plane)?.city) && c.owner === v.me);
      const plane = (v.aircraft ?? []).find((a) => a.id === s.air.plane && a.owner === v.me);
      const pick = city && plane ? pickerCandidates(v, city, plane, s.air.what, s.orders[s.seat] ?? []).find((c) => hexEq(c.at, h)) : undefined;
      if (plane && pick?.peaceTeam !== undefined && !v.finished) {
        // A team at Peace: ask about War first, as for a ground unit's shot; the strike goes in with the declaration.
        s.set({ air: { plane: null, what: null, hover: null } });
        promptWar(pick.peaceTeam, plane.id, h, true, true);
        return;
      }
      if (plane && pick && !v.finished) {
        s.editOrders((ord) => setAirOrder(ord, s.air.what === 'strike' ? { type: 'AirStrike', plane: plane.id, target: h } : s.air.what === 'recon' ? { type: 'AirRecon', plane: plane.id, target: h } : { type: 'Rebase', plane: plane.id, to: pick.city!.id }), 'commit');
        s.set({ air: { plane: null, what: null, hover: null } });
        return;
      }
    }
    // A selected city's improvable tile (with none of your units on it): the click picks it, and its
    // row in the Improvements group lights up.
    if (s.selection?.kind === 'city') {
      const own = v.cities.find((c) => c.id === s.selection!.id && c.owner === v.me);
      const standing = v.units.some((u) => hexEq(u.pos, h) && u.owner === v.me);
      if (!standing && own?.detail?.options.some((o) => o.item.kind === 'Improvement' && hexEq(o.item.tile, h))) {
        s.set({ improve: { ...s.improve, pick: h } });
        feel.select();
        return;
      }
    }
    const here = v.units.filter((u) => hexEq(u.pos, h));
    const city = v.cities.find((c) => hexEq(c.pos, h));
    // Cycle: own units on the tile, then the city (a foreign city's card before its garrison), then other units.
    const unitCands = (own: boolean) => here.filter((u) => (u.owner === v.me) === own)
      .sort((a, b) => Number(b.military) - Number(a.military)).map((u) => ({ kind: 'unit' as const, id: u.id }));
    const cands: ({ kind: 'unit' | 'air' | 'city'; id: number })[] = [
      ...unitCands(true),
      ...(city && city.owner === v.me ? (v.aircraft ?? []).filter((a) => a.city === city.id).map((a) => ({ kind: 'air' as const, id: a.id })) : []),
      ...(city ? [{ kind: 'city' as const, id: city.id }] : []),
      ...unitCands(false),
    ];
    if (!cands.length) {
      s.select(null);
      return;
    }
    const cur = s.selection;
    const idx = cur ? cands.findIndex((c) => c.kind === cur.kind && c.id === cur.id) : -1;
    s.select(cands[(idx + 1) % cands.length]);
    feel.select();
  };

  const confirmOrder = (order: TapOrder) => {
    const s = useGame.getState();
    const u = selectedOwnUnit();
    const v = s.views?.[s.seat];
    if (!u || !v || s.busy || !sameOrderContext(order, { unit: u.id, seat: s.seat, turn: v.turn, mode: s.mode })) return;
    clearTouch();
    // The same command and validation pipeline as desktop right-click, including ranged mode.
    moveSelected(order.target);
  };

  const touchHex = (point: Point, context: boolean) => {
    const hex = targetAtScreen(point.x, point.y);
    const s = useGame.getState();
    const v = s.views?.[s.seat];
    if (!v || s.busy || tileIndex(hex, v.width, v.height) < 0) { clearTouch(); return; }
    const unit = selectedOwnUnit();
    // Touch selects another own unit, but an empty own city remains a move destination.
    const selectsUnit = pointerIntent(v, unit, hex, s.mode) === 'select'
      && v.units.some((u) => u.owner === v.me && u.id !== unit?.id && hexEq(u.pos, hex));
    const result = selectsUnit ? { action: 'select' as const }
      : tapOrder(tapPreview, unit ? { unit: unit.id, seat: s.seat, turn: v.turn, mode: s.mode } : null, unit?.pos ?? null, hex);
    setHover(null);
    if (result.action !== 'select') {
      lastTap.current = null;
      if (result.action === 'confirm' && !context) confirmOrder(result.order);
      else {
        if (!context) feel.preview();
        setTapPreview(result.order);
        setTouchInfo({ hex, context, placement: point.y > size.current.h / 2 ? 'top' : 'bottom' });
      }
      return;
    }
    const tap = { point, time: performance.now() };
    if (!context && isDoubleTap(lastTap.current, tap)) {
      zoomAt(point.x, point.y, 1.5);
      clearTouch();
      lastTap.current = null;
      return;
    }
    lastTap.current = context ? null : tap;
    setTapPreview(null);
    if (s.mode === 'normal' && lens === 'settle' && siteAt(hex)) {
      // Inspect occupied sites too, without cycling away from the selected Settler.
      setTouchInfo({ hex, context, placement: point.y > size.current.h / 2 ? 'top' : 'bottom' });
      return;
    }
    // Long-press inspects without executing a ranged order or cycling away from the selected unit.
    if (!context) clickHex(hex, false);
    setTouchInfo(context || !phone ? { hex, context, placement: point.y > size.current.h / 2 ? 'top' : 'bottom' } : null);
  };

  const stepGesture = (event: GestureEvent) => {
    const result = gestureStep(gesture.current, event, cam.current, size.current);
    gesture.current = result.state;
    // Only a camera that moved needs a redraw: a plain mouse move over the map changes nothing drawn by itself.
    const c = cam.current;
    if (result.camera.x !== c.x || result.camera.y !== c.y || result.camera.zoom !== c.zoom) dirty.current = true;
    Object.assign(c, result.camera);
    if (result.state.phase !== 'pending' || result.state.contacts.length !== 1) {
      if (holdTimer.current) clearTimeout(holdTimer.current);
    }
    if (result.state.phase === 'pan' || result.state.phase === 'pinch') {
      lastTap.current = null;
      clearTouch();
      setHover(null);
    }
    if (result.state.phase === 'path') {
      const point = result.state.contacts[0]?.point;
      if (point) setDragTarget(hexAtScreen(point.x, point.y));
    }
    const intent = result.intent;
    if (!intent) return;
    if (intent.type === 'context') {
      feel.longPress();
      touchHex(intent.point, true);
    }
    else if (intent.type === 'path') {
      setDragTarget(null);
      moveSelected(hexAtScreen(intent.point.x, intent.point.y));
    } else if (intent.kind === 'mouse') {
      clearTouch();
      const hex = targetAtScreen(intent.point.x, intent.point.y);
      if (mode === 'normal' && lens === 'settle' && siteAt(hex)) {
        setTouchInfo({ hex, context: false, placement: intent.point.y > size.current.h / 2 ? 'top' : 'bottom' });
      } else clickHex(hex);
    } else touchHex(intent.point, false);
  };

  const onPointerDown = (e: React.PointerEvent<HTMLCanvasElement>) => {
    cancelAnimationFrame(focusAnimation.current);
    canvasRef.current?.focus({ preventScroll: true });
    lastPointerType.current = e.pointerType || 'mouse';
    const point = local(e);
    e.currentTarget.setPointerCapture(e.pointerId);
    const u = selectedOwnUnit();
    stepGesture({ type: 'down', id: e.pointerId, point, kind: lastPointerType.current, button: e.button,
      pathDrag: e.button === 0 && !!u && hexEq(u.pos, hexAtScreen(point.x, point.y)) && useGame.getState().mode === 'normal' });
    if (holdTimer.current) clearTimeout(holdTimer.current);
    if (lastPointerType.current !== 'mouse' && gesture.current.contacts.length === 1) {
      holdTimer.current = setTimeout(() => stepGesture({ type: 'hold' }), LONG_PRESS_MS);
    }
  };

  // The map re-renders (and redraws) only when the hovered tile or city banner changes, never for each pixel the mouse
  // moves inside them: on a crowded late-game map that per-pixel work made hover and clicks lag. The banner tooltip
  // follows the pointer by moving its element directly.
  const bannerAt = (point: Point) =>
    bannerHits.current.findLast(({ rect }) => point.x >= rect.x && point.x <= rect.x + rect.w && point.y >= rect.y && point.y <= rect.y + rect.h)?.city.id ?? null;
  const updateMouseHover = (point: Point) => {
    const h = targetAtScreen(point.x, point.y);
    if (!hover || !hexEq(hover.hex, h) || bannerAt({ x: hover.sx, y: hover.sy }) !== bannerAt(point)) {
      setHover({ hex: h, sx: point.x, sy: point.y });
      return;
    }
    const tip = bannerTipRef.current;
    if (tip) {
      tip.style.left = `${bannerTipLeft(point.x)}px`;
      tip.style.top = `${bannerTipTop(point.y)}px`;
    }
  };

  const onPointerMove = (e: React.PointerEvent) => {
    const point = local(e);
    stepGesture({ type: 'move', id: e.pointerId, point });
    if ((e.pointerType || 'mouse') === 'mouse') updateMouseHover(point);
  };

  const onPointerUp = (e: React.PointerEvent<HTMLCanvasElement>) => {
    if (holdTimer.current) clearTimeout(holdTimer.current);
    stepGesture({ type: 'up', id: e.pointerId, point: local(e) });
    if (e.currentTarget.hasPointerCapture?.(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
  };

  const onPointerCancel = (e: React.PointerEvent) => {
    if (holdTimer.current) clearTimeout(holdTimer.current);
    stepGesture({ type: 'cancel', id: e.pointerId });
    lastTap.current = null;
    setDragTarget(null);
    clearTouch();
  };

  const onContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    // iPad/Android may synthesize contextmenu after a hold; only a real mouse right-click commits.
    if (lastPointerType.current !== 'mouse' || ('pointerType' in e.nativeEvent && e.nativeEvent.pointerType === 'touch')) return;
    clearTouch();
    const p = local(e);
    moveSelected(targetAtScreen(p.x, p.y));
  };

  const onWheel = (e: React.WheelEvent) => {
    const p = local(e);
    // Trackpad pinch arrives as ctrl+wheel with small deltas; mouse wheels as larger steps.
    const k = e.ctrlKey ? 0.012 : 0.0018;
    zoomAt(p.x, p.y, Math.exp(-e.deltaY * k));
  };

  useEffect(() => {
    const c = canvasRef.current!;
    const prevent = (e: WheelEvent) => e.preventDefault();
    c.addEventListener('wheel', prevent, { passive: false });
    return () => c.removeEventListener('wheel', prevent);
  }, []);

  const hoverTile = view && hover ? view.tiles[tileIndex(hover.hex, view.width, view.height)] : null;
  const hoveredBanner = hover && lastPointerType.current === 'mouse' && !gesture.current.contacts.length
    ? bannerHits.current.findLast(({ rect }) => hover.sx >= rect.x && hover.sx <= rect.x + rect.w && hover.sy >= rect.y && hover.sy <= rect.y + rect.h)
    : undefined;
  const sidebarLeft = () => raceRect.current && canvasRef.current
    ? raceRect.current.left - canvasRef.current.getBoundingClientRect().left : size.current.w;
  const bannerTipLeft = (sx: number) => Math.min(Math.max(8, sx + 12), Math.max(8, sidebarLeft() - 328));
  const bannerTipTop = (sy: number) => Math.min(sy + 16, size.current.h - 64);
  // Settle lens: explored land that is not tinted says why (the core's rule, not a local copy), so no legal-looking tile is a mystery.
  const blockedSiteNote = useMemo(() => {
    if (lens !== 'settle' || !view || !hover || !hoverTile?.explored || !hoverTile.terrain || !scoreUnit || siteAt(hover.hex)) return undefined;
    return settleTileNote(view, hover.hex);
  }, [lens, view, hover?.hex.q, hover?.hex.r, hoverTile, scoreUnit, sitesByHex]);
  // Cards sit beside the hovered hex, clear of the selected unit and the docked panels.
  const toScreen = (h: Hex) => {
    const p = hexToPixel(h, HEX_SIZE);
    const c = cam.current;
    return { x: (p.x - c.x) * c.zoom + size.current.w / 2, y: (p.y - c.y) * c.zoom + size.current.h / 2 };
  };
  const peaceHere = !hover || !view ? null : intent === 'peaceful' ? peacefulAt(view, hover.hex)
    : mode === 'ranged' && ownSel?.military ? rangedPeaceAt(view, hover.hex) : null;
  const cardAnchor: CardAnchor | null = hover ? (() => {
    const at = toScreen(hover.hex);
    const r = HEX_SIZE * cam.current.zoom;
    const avoid = [...panels.current, ...banners.current].map((p) => ({ left: p.x, top: p.y, right: p.x + p.w, bottom: p.y + p.h }));
    if (ownSel) {
      const u = toScreen(ownSel.pos);
      avoid.unshift({ left: u.x - r, top: u.y - r, right: u.x + r, bottom: u.y + r });
    }
    return { x: at.x, y: at.y, r, avoid };
  })() : null;

  return (
    <div ref={wrapRef} className="map-wrap" onPointerDownCapture={(e) => { lastPointerType.current = e.pointerType || 'mouse'; }}>
      <canvas
        ref={canvasRef}
        tabIndex={0}
        data-game-map
        data-overlay-slot="map-viewport"
        aria-label="World map. Drag to pan, pinch to zoom. Tap a destination twice to order; hold for actions. Arrow keys or WASD pan. Return selects the next decision, or ends the turn when none remain. Space skips the selected unit this turn. Shift+Return requests End Turn. F fortifies military units or makes civilians wait. H heals a damaged unit until it is whole. Use Cities seen to select known cities."
        className={`map-canvas ${mode === 'ranged' ? 'is-ranged' : ''} ${intent === 'select' ? 'is-select' : ''} ${intent === 'peaceful' ? 'is-peaceful' : ''}`}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        // Safari/WKWebView can deliver a mouse move without a matching pointer move.
        onMouseMove={(e) => { if (!gesture.current.contacts.length) updateMouseHover(local(e)); }}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerCancel}
        onLostPointerCapture={(e) => { if (gesture.current.contacts.some((p) => p.id === e.pointerId)) onPointerCancel(e); }}
        onPointerLeave={() => setHover(null)}
        onMouseLeave={() => setHover(null)}
        onContextMenu={onContextMenu}
        onWheel={onWheel}
      />
      <canvas ref={flightRef} className="counter-flight-layer" data-overlay-slot="counter-flight" aria-hidden="true" />
      {!touchInfo && hoveredBanner && view && <div className="map-banner-tip" role="tooltip" data-testid="banner-tooltip"
        ref={bannerTipRef} style={{ left: bannerTipLeft(hover!.sx), top: bannerTipTop(hover!.sy) }}>
        {bannerTooltip(view, hoveredBanner.city)}
      </div>}
      <CompassRose ref={compassRef} />
      <LensSwitcher players={view?.players ?? []} me={view?.me} visibleBarbarians={!!view?.units.some((u) => isBarbarian(u.owner))} supply={view?.supply} era={view?.research.era} lens={lens} empty={!lensTiles.length} overlayEmpty={{ resources: !!view && overlayEmpty(view, 'resources', overlayTiles), borders: !!view && overlayEmpty(view, 'borders', overlayTiles) }} collapsed={!!selection} onChange={(next) => { clearTouch(); setLens(next); }} sites={sites} settlerId={settleUnit?.id ?? null} onInspect={(hex) => {
        useGame.getState().jumpTo(hex);
        setHover(null);
        setTapPreview(null);
        setTouchInfo({ hex, context: false, placement: 'bottom' });
      }} />
      {touchInfo && view && (
        <section ref={inspectorRef} className={`map-touch-card at-${touchInfo.placement}`} data-settlement={!!inspectedSite} data-overlay-slot="map-preview" aria-label={touchInfo.context ? 'Tile actions' : 'Destination preview'}>
          <div className="map-touch-head">
            {/* Attacks name their target ("Attack Sunken India"), never raw coordinates. */}
            <b>{inspectedSite && !activePreview ? 'Place for a city' : <>{activePreview ? mode === 'ranged' ? 'Ranged strike' : pathAttack || previewPeace ? 'Attack' : 'Move' : 'Tile'} {(activePreview && (mode === 'ranged' || pathAttack || previewPeace) && forecastTargetName(view, touchInfo.hex)) || coord(touchInfo.hex)}</>}</b>
            <button className="btn quiet" aria-label="Close tile actions" onClick={clearTouch}>Close</button>
          </div>
          {inspectedSite && <div className="settlement-details" aria-label="Settlement suitability"><SettlementDetails faction={view?.players.find((p) => p.id === view.me)?.faction} key={`${inspectedSite.pos.q},${inspectedSite.pos.r}`} site={inspectedSite} /></div>}
          {siteAt(touchInfo.hex) && ownSel && isFounderCandidate(view, ownSel) && !activePreview && !hexEq(ownSel.pos, touchInfo.hex) && <button className="btn primary" onClick={() => {
            setTapPreview({ unit: ownSel.id, seat, turn: view.turn, mode, target: touchInfo.hex });
          }}>Preview move to this site</button>}
          {activePreview && previewPeace && <>
            <p className="peace-hint-text">{relationWith(view, previewPeace.team)?.state === 'Alliance' ? <><b>Allied with {empireTitle(view, previewPeace.team)}.</b> Declaring War on an ally is a betrayal.</> : <><b>At Peace with {empireTitle(view, previewPeace.team)}.</b> Declare War to attack.</>}</p>
            <button className="btn primary confirm-order" onClick={() => confirmOrder(activePreview)}>Declare War…</button>
          </>}
          {activePreview && !previewPeace && <>
            {/* The forecast's headline sits where you confirm; its full breakdown follows below. */}
            {forecast && ownSel && <ForecastSummary forecast={forecast} ranged={forecast.ranged} view={view} unit={ownSel} target={activePreview.target} assault={assault} />}
            {mode !== 'ranged' && routeWarningAt(activePreview.target) && <p className={isCalmRouteNote(routeWarningAt(activePreview.target)) ? 'route-note' : 'route-warning'} role={isCalmRouteNote(routeWarningAt(activePreview.target)) ? undefined : 'alert'} data-testid="route-warning">{routeWarningAt(activePreview.target)}</p>}
            {settlePreviewBlocker && <p className="route-warning" role="status" data-testid="settle-preview-blocker">{settlePreviewBlocker}</p>}
            <p>{blockedShot ?? (mode === 'ranged' ? `Range ${ownSel?.range}` : pathInfo?.unreachable ? 'No known path to this tile.' : `${pathInfo?.steps && ownSel && previewTarget ? [routeTurnsText(pathInfo.steps, ownSel.pos, previewTarget), roadBonusNote(view, pathInfo.steps, ownSel.pos, ownSel.max_moves)].filter(Boolean).join(' ') : '1 turn path.'} Orders resolve when everyone is ready.`)}</p>
            <button className="btn primary confirm-order" disabled={!!blockedShot} title={blockedShot ?? undefined} onClick={() => confirmOrder(activePreview)}>Confirm {mode === 'ranged' || pathAttack ? 'attack' : 'move'}</button>
            {!blockedShot && <p className="muted">{lastPointerType.current === 'mouse' ? `Or right-click to ${mode === 'ranged' || pathAttack ? 'attack' : 'move'} here.` : 'Or tap this tile again to confirm.'}</p>}
          </>}
          {forecast && activePreview && ownSel && (shotForecast
            ? <DualForecastCard shot={shotForecast} charge={forecast} view={view} unit={ownSel} target={activePreview.target} />
            : <ForecastCard forecast={forecast} ranged={forecast.ranged} view={view} unit={ownSel} target={activePreview.target} bowNote={shotWhy ?? undefined} assault={assault} />)}
          {!inspectedSite && <TileCard view={view} hex={touchInfo.hex} reinforcement={reinforcementAt(touchInfo.hex)}
            note={lens === 'settle' && scoreUnit && view.tiles[tileIndex(touchInfo.hex, view.width, view.height)]?.explored
              ? settleTileNote(view, touchInfo.hex) : undefined} />}
          {(touchInfo.context || view.units.some((u) => hexEq(u.pos, touchInfo.hex)) || view.cities.some((c) => hexEq(c.pos, touchInfo.hex))) && <div className="map-context-actions">
            <button className="btn" onClick={() => { useGame.getState().setMode('normal'); clickHex(touchInfo.hex); clearTouch(); }}>Select here</button>
            {ownSel && hexEq(ownSel.pos, touchInfo.hex) && <>
        {isFounderCandidate(view, ownSel) && <button className="btn" disabled={!!foundCityUnitBlocker(view, ownSel)} title={foundCityUnitBlocker(view, ownSel) ?? (ownSel.kind !== 'Settler' ? 'Consumes this veteran; combat can wound it before founding' : undefined)} onClick={() => { foundCitySelected(); clearTouch(); }}>Found City</button>}
              <button className="btn" onClick={() => { sleepSelected(); clearTouch(); }}>{ownSel.military && ownSel.kind !== 'Scout' ? 'Fortify' : 'Wait'}</button>
              <button className="btn" onClick={() => { skipSelected(); clearTouch(); }}>Skip this turn</button>
              {ownSel.ranged > 0 && <button className="btn" onClick={() => { toggleRangedMode(); clearTouch(); }}>Ranged Attack</button>}
              <button className="btn" onClick={() => { cancelSelected(); clearTouch(); }}>Cancel order</button>
            </>}
          </div>}
        </section>
      )}
      {!touchInfo && peaceHere && <PeaceHint anchor={cardAnchor!} team={empireTitle(view!, peaceHere.team)} mouse={lastPointerType.current === 'mouse'} allied={relationWith(view!, peaceHere.team)?.state === 'Alliance'} />}
      {!touchInfo && forecast && hover && ownSel && view && (
        shotForecast
          ? <DualForecastCard shot={shotForecast} charge={forecast} anchor={cardAnchor!} view={view} unit={ownSel} target={hover.hex} routeWarning={routeWarningAt(hover.hex)} />
          : <ForecastCard forecast={forecast} anchor={cardAnchor!} ranged={forecast.ranged} view={view} unit={ownSel} target={hover.hex} routeWarning={forecast.ranged ? undefined : routeWarningAt(hover.hex)} bowNote={shotWhy ?? undefined} assault={assault} />
      )}
      {!touchInfo && !forecast && airForecast && airShot && hover && view && (
        <ForecastCard forecast={airForecast} anchor={cardAnchor!} ranged view={view} unit={airShot.unit} target={hover.hex} />
      )}
      {/* One card at a time: the Peace hint is the decision here, so the tile card waits (both used the same spot). */}
      {!touchInfo && !hoveredBanner && !forecast && !airForecast && !peaceHere && hover && hoverTile && (hoverTile.explored || (view?.units.some((u) => u.heir_of != null && hexEq(u.pos, hover.hex)) ?? false)) && tileHoverReady && view && !gesture.current.contacts.length && (
        inspectedSite ? <SettlementCard site={inspectedSite} anchor={cardAnchor!} faction={view?.players.find((p) => p.id === view.me)?.faction} /> : <TileCard view={view} hex={hover.hex} anchor={cardAnchor!} blockedReason={blockedShot || undefined} routeWarning={ownSel ? routeWarningAt(hover.hex) : undefined} reinforcement={reinforcementAt(hover.hex)} note={placeNote ?? blockedSiteNote} refusal={placeRefusal} />
      )}
    </div>
  );
}

function clampCamera(c: Camera, view: PlayerView, w: number, h: number) {
  const b = worldBounds(view);
  const halfW = w / 2 / c.zoom;
  const halfH = h / 2 / c.zoom;
  const clamp = (v: number, lo: number, hi: number) => (lo > hi ? (lo + hi) / 2 : Math.max(lo, Math.min(hi, v)));
  // Allow panning a bit past the edge so edge tiles can be centered under panels.
  c.x = clamp(c.x, b.minX - halfW * 0.6 + 40, b.maxX + halfW * 0.6 - 40);
  c.y = clamp(c.y, b.minY - halfH * 0.6 + 40, b.maxY + halfH * 0.6 - 40);
}
