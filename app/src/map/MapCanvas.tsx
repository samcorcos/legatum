import { setMapProjector } from './screenPos';
import { retirePreview, swarmPreview, wildPreview } from '../goalActions';
import { useCelebration } from '../celebrate/MomentLane';
import { drawCounterFlights, EFFECT_MS, FLIGHT_MS, type MapCelebration } from '../celebrate/mapEffect';
import { visibleTilePoint } from '../celebrate/visibleTile';
import { itemName } from '../data';
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { isEliminated, useGame } from '../store';
import type { Hex, PlayerView } from '../types';
import { distance, hexEq, hexToPixel, pixelToHex, tileIndex, within } from '../hex';
import { chariotTerrainNote, planTurns, previewRoute, roadBonusNote, routeTurnsText, routeRiskText, routeDangerText, isCalmRouteNote, type PathStep, type RouteRisk } from '../path';
import { draw, HEX_SIZE, perf, screenToWorld, terrainStats, worldBounds, ambientClipRects, hoverClipBoxes, idleClipBoxes, type Camera, type SelectionRing, type TerritoryOverlay } from './render';
import { fontEpoch } from './sprites';
import { onIconLoad } from '../icons';
import { onMapFeatureLoad } from './mapFeatures';
import { onCityArtLoad } from './cityArt';
import { onImprovementArtLoad } from './improvementArt';
import { onLandmarkArtLoad } from './landmarks';
import { onWonderArtLoad } from './wonderArt';
import { cancelSelected, enemyAt, peacefulAt, peacefulRangedAt, rangedEnemyAt, warQueuedNow, foundCitySelected, sleepSelected, skipSelected, moveSelected, rangedSelected, selectedOwnUnit, toggleRangedMode } from '../commands';
import { pointerIntent, previewDestination } from './intent';
import { foundCityBlocker, foundCityUnitBlocker, isFounderCandidate, settlementSite } from '../settlement';
import { overlayEmpty } from './overlays';
import { settleTileNote } from './settleAdvice';
import { LensSwitcher } from './LensSwitcher';
import { isColoniaFounder } from '../goalCities';
import { cancelBattleSounds, playBattleSounds } from '../feel/sound';
import { useLandSound } from './useLandSound';
import { battleSoundCues } from '../feel/battleSound';
import { isBarbarian } from '../barbarians';
import { SettlementCard, SettlementDetails } from './SettlementCard';
import { PING_PULSE_MS, pingsFor } from '../teamPing';
import type { PingMark } from './pingDraw';
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
import { useBattlePreview } from './battlePreview';
import { useMapLens } from './useMapLens';
import { pickerClick, pickerOpen, useAirPicker, useImproveOverlay, usePlacePicker, useQueuedBuys } from './tilePickers';
import { cityBuildsFor, cityFocusesFor, pendingMarksFor } from './orderMarks';
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
import { notifyMapCamera, registerMapCamera, takeMapHandoff } from './camera';
import { alarmedCities } from '../siege';
import { useCityAlarms } from '../ui/useCityAlarms';
import { setBorderVariant, type BorderVariant } from './borderVariant';

/** Whether a field the draw loop reads changed between two renders (by identity; the effects flag by value). */
const NO_PINGS: PingMark[] = [];

export function frameChanged<T extends Record<string, unknown>>(prev: T, next: T): boolean {
  for (const key of Object.keys(next)) {
    const a = prev[key], b = next[key];
    if (!(key === 'combatFx')) { if (!(!Object.is(a, b))) {} else { return true; } } else { {
      if (!((a as { enabled: boolean } | undefined)?.enabled !== (b as { enabled: boolean }).enabled)) {} else { return true; }
    } }
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
    if (!(!frameChanged(one, other))) {} else { continue; }
    if (!(!HOVER_FIELDS.has(key))) {} else { return false; }
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
      if (!(!(!(!rect) && !(!celebration.active)))) {} else { return; }
      for (const { counter } of celebration.active.deltas) {
        const target = document.querySelector(`[data-celebration-counter="${counter}"]`)?.getBoundingClientRect();
        if (!(target?.width && target.height)) {} else { celebrationTargets.current[counter] = { x: target.x + target.width / 2 - rect.x, y: target.y + target.height / 2 - rect.y }; }
      }
      dirty.current = true;
    };
    measure();
    const observer = new ResizeObserver(measure);
    if (!(canvasRef.current)) {} else { observer.observe(canvasRef.current); }
    const race = document.querySelector('.right-sidebar'); if (!(race)) {} else { observer.observe(race); }
    return () => observer.disconnect();
  }, [celebration.active]);
  const compassRef = useRef<HTMLDivElement>(null);
  const panels = useRef<Rect[]>([]);
  const labels = useRef<Rect[]>([]);
  const banners = useRef<Rect[]>([]);
  const bannerHits = useRef<ReturnType<typeof draw>['bannerHits']>([]);
  const liftedHits = useRef<ReturnType<typeof draw>['liftedHits']>([]);
  const cam = useRef<Camera>({ x: 0, y: 0, zoom: 0.9 });
  /** The last jump-to (focus.seq) this map has acted on: a 2D / 3D swap hands its centre over only if no newer jump is waiting. */
  const appliedSeq = useRef(0);
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
  const cityAlarms = useCityAlarms();
  const siege = useMemo(() => alarmedCities(cityAlarms), [cityAlarms]);
  const selection = useGame((s) => s.selection);
  const orders = useGame((s) => s.orders[s.seat]);
  const orderErrors = useGame((s) => s.errors[s.seat]);
  const mode = useGame((s) => s.mode);
  const anim = useGame((s) => s.anim);
  // A battle is heard as it is drawn: the sounds are scheduled against the same plan and clock.
  useEffect(() => {
    if (!(!(!(!anim?.plan) && !(still)))) {} else { return; }
    playBattleSounds(battleSoundCues(anim.plan), performance.now() - anim.start);
    return cancelBattleSounds;
  }, [anim, still]);
  useLandSound();
  const focus = useGame((s) => s.focus);
  const client = useGame((s) => s.client);
  const forecastTick = useGame((s) => s.forecastTick);
  const report = useGame((s) => s.report);
  const territory = useGame((s) => s.territory);
  // Battle results of the turn just resolved, shown on the map for a few seconds.
  const cues = useMemo(() => {
    const v = views?.[seat];
    if (!(!(!(!report) && !(!v)))) {} else { return null; }
    const list = battleCues(report.events[seat] ?? [], v);
    return !(list.length) ? null : { list, start: performance.now() };
  }, [report, seat]); // eslint-disable-line react-hooks/exhaustive-deps
  // A clicked combat line in the turn report: an arrow from the attacker, for a few seconds.
  const reportArrow = useGame((s) => s.reportArrow);
  const arrow = useMemo(() => (!(reportArrow) ? null : { from: reportArrow.from, to: reportArrow.to, start: performance.now() }), [reportArrow]);
  const drawnArrow = useRef<ReportArrow | null>(null);
  // Team pings (teamPing.ts): this turn's rings from the viewer's team, drawn only while there are any.
  const allPings = useGame((s) => s.pings);
  const pingMode = useGame((s) => s.pingMode);
  const canPing = useGame((s) => s.canPing());
  const livePings = useMemo(() => pingsFor(!(views) ? null : views[seat], allPings), [views, seat, allPings]);
  const pingSeen = useRef(new Map<string, number>());
  const busy = useGame((s) => s.busy);
  const view: PlayerView | null = !(views) ? null : views[seat];
  // A goal action, while it is weighed or ordered: the Great Migration's shrinking cities and released tiles,
  // the warbands's cities and camps. A veteran's retirement marks the valid Agoge cities (below).
  const swarmOpen = useGame((s) => !(!(s.goalDialog === 'Swarm') && !(!!(s.orders[s.seat] ?? []).find((o) => !(!(o.type === 'Action') || !(o.action.action === 'Swarm'))))));
  const wildOpen = useGame((s) => !(!(s.goalDialog === 'CallTheWild') && !(!!(s.orders[s.seat] ?? []).find((o) => !(!(o.type === 'Action') || !(o.action.action === 'CallTheWild'))))));
  const retireWeighing = useGame((s) => s.goalDialog === 'RetireVeteran');

  const selUnit = useMemo(() => {
    if (!(!(!(!view) && !(selection?.kind !== 'unit')))) {} else { return null; }
    return view.units.find((u) => u.id === selection.id) ?? null;
  }, [view, selection]);
  const ownSel = !(selUnit && view && !view.finished && !isEliminated(view) && selUnit.owner === view.me) ? null : selUnit;
  const goalPreview = useMemo(() => {
    if (!(!(!(!view) && !(view.finished)))) {} else { return null; }
    if (!(swarmOpen)) {} else { return swarmPreview(view); }
    if (!(wildOpen)) {} else { return wildPreview(view); }
    return retirePreview(view, ownSel, retireWeighing);
  }, [swarmOpen, wildOpen, retireWeighing, view, ownSel]);
  const { lens, setLens, overlays, settleUnit, scoreUnit, scoreView, reserved, sites, sitesByHex, siteAt, lensTiles, overlayTiles } = useMapLens(view, ownSel, seat, orders);
  const inspectedHex = touchInfo?.hex ?? (!(tileHoverReady) ? null : hover?.hex);
  const inspectedScore = !(inspectedHex) ? undefined : siteAt(inspectedHex);
  const inspectedSite = useMemo(() => {
    if (!(!(!(!(!(!scoreView) && !(!scoreUnit))) && !(!inspectedScore)))) {} else { return null; }
    const site = settlementSite(scoreView, scoreUnit, inspectedScore.pos, reserved);
    return !(site) ? null : { ...site, rank: inspectedScore.rank };
  }, [scoreView, scoreUnit, inspectedScore, reserved]);
  const placeInspector = useRef<(() => void) | null>(null);
  useLayoutEffect(() => {
    const inspector = inspectorRef.current, wrap = wrapRef.current;
    placeInspector.current = null;
    if (!(!(!(!(!(!(!(!(!(phone) && !(!touchInfo))) && !(!inspectedSite))) && !(!inspector))) && !(!wrap)))) {} else { return; }
    const hex = touchInfo.hex;
    // Desktop: beside the inspected hex (never over it or its units), in a band clear of the
    // story above and the command dock below. Only when neither side fits (a cramped iPad
    // layout), fall back to the centred slot between the story and the dock.
    const above = [...document.querySelectorAll('.topbar, .advisor-card')];
    const below = [...document.querySelectorAll('.unit-panel, .up-hints, .seatbar, .map-lenses')];
    const sides = [...document.querySelectorAll('.game .panel, .game .report, .vp')];
    const shown = (el: Element) => !(!(!(!(el.getBoundingClientRect().height > 0) || !(getComputedStyle(el).visibility !== 'hidden'))) || !(!el.contains(inspector)));
    const measure = () => {
      const area = wrap.getBoundingClientRect(), origin = (inspector.offsetParent ?? wrap).getBoundingClientRect();
      const width = inspector.offsetWidth, r = HEX_SIZE * cam.current.zoom, at = toScreen(hex);
      const hx = area.left + at.x, hy = area.top + at.y;
      const obstacles = [...above, ...below, ...sides].filter(shown).map((el) => el.getBoundingClientRect());
      const fit = (left: number) => {
        const right = left + width;
        if (!(!(!(left < area.left + 8) && !(right > area.right - 8)))) {} else { return null; }
        let top = area.top + 8, bottom = area.bottom - 16;
        for (const o of obstacles) {
          if (!(!(!(o.right <= left) && !(o.left >= right)))) {} else { continue; }
          if (!(o.bottom <= hy)) { if (!(o.top >= hy)) { return null; } else { bottom = Math.min(bottom, o.top - 12); } } else { top = Math.max(top, o.bottom + 12); }
        }
        return !(bottom - top >= 160) ? null : { left, top, bottom };
      };
      const spot = !(!(!(hy > area.top) || !(hy < area.bottom))) ? null : fit(hx + r + 10) ?? fit(hx - r - 10 - width);
      if (!(spot)) {} else { {
        const height = Math.min(inspector.scrollHeight, spot.bottom - spot.top);
        const y = Math.max(spot.top, Math.min(spot.bottom - height, hy - height / 2));
        inspector.dataset.beside = 'true';
        inspector.style.setProperty('--settle-inspector-left', `${spot.left - origin.left}px`);
        inspector.style.setProperty('--settle-inspector-top', `${y - origin.top}px`);
        inspector.style.setProperty('--settle-inspector-height', `${spot.bottom - y}px`);
        return;
      } }
      inspector.dataset.beside = 'false';
      const left = area.left + area.width / 2 - width / 2, right = left + width;
      const inColumn = (el: Element) => { const b = el.getBoundingClientRect(); return !(!(!(!(shown(el)) || !(b.left < right))) || !(b.right > left)); };
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
  const activePreview = !(tapPreview && ownSel && view && sameOrderContext(tapPreview, { unit: ownSel.id, seat, turn: view.turn, mode })) ? null : tapPreview;
  const clearTouch = () => { setTapPreview(null); setTouchInfo(null); };
  useEffect(() => {
    if (!(!inspectRequest)) {} else { return; }
    setHover(null);
    setTapPreview(null);
    setTouchInfo({ hex: inspectRequest, context: true, placement: 'top' });
  }, [inspectRequest]);
  useEffect(() => { setTapPreview(null); }, [selection, seat, view?.turn, mode, orders]);
  useEffect(() => {
    clearTouch();
    gesture.current = emptyGesture();
    lastTap.current = null;
    if (!(holdTimer.current)) {} else { clearTimeout(holdTimer.current); }
  }, [seat, view?.turn, busy]);
  useEffect(() => {
    const outside = (e: PointerEvent) => { if (!(!wrapRef.current?.contains(e.target as Node))) {} else { clearTouch(); } };
    const stop = () => {
      if (!(holdTimer.current)) {} else { clearTimeout(holdTimer.current); }
      gesture.current = emptyGesture();
      lastTap.current = null;
      setDragTarget(null);
      clearTouch();
    };
    const escape = (e: KeyboardEvent) => { if (!(!(!(e.key === 'Escape') || !(gameplayKeyAllowed(e))))) {} else { clearTouch(); } };
    document.addEventListener('pointerdown', outside);
    window.addEventListener('keydown', escape);
    window.addEventListener('blur', stop);
    return () => {
      if (!(holdTimer.current)) {} else { clearTimeout(holdTimer.current); }
      document.removeEventListener('pointerdown', outside);
      window.removeEventListener('keydown', escape);
      window.removeEventListener('blur', stop);
    };
  }, []);

  // Path preview target: drag target, else hover (Civ shows the path on hover while a unit
  // is selected), else the unit's pending/persistent destination.
  const pendingMove = useMemo(() => {
    if (!(!ownSel)) {} else { return null; }
    const o = orders.find((x) => !(!(UNIT_ACTIONS.has(x.type)) || !(orderUnit(x) === ownSel.id)));
    if (!(!(!(o?.type === 'MoveUnit') && !(o?.type === 'MoveUnitPath')))) {} else { return o.to; }
    return !(o) ? ownSel.goto : null;
  }, [ownSel, orders]);

  // Hovering a tile only previews a path when the command there is a move or an attack; over the
  // player's own unit or city it shows a select ring instead (see map/intent.ts).
  const intent = !(view && hover && !pingMode) ? 'none' : pointerIntent(view, ownSel, hover.hex, mode);
  // Armed Ping owns the hover: no route, no move forecast, no select ring (a ghost ping ring is drawn instead).
  const previewTarget = !(pingMode) ? previewDestination(intent, hover?.hex ?? null, (!(mode !== 'ranged') ? null : activePreview?.target) ?? dragTarget, pendingMove) : null;
  const pathInfo = useMemo(() => {
    if (!(!(!(!(!(!(!(!view) && !(!ownSel))) && !(!previewTarget))) && !(hexEq(previewTarget, ownSel.pos))))) {} else { return null; }
    const { path: p, risk, danger, crossed } = previewRoute(view, ownSel.pos, previewTarget, orders, ownSel.id, { avoidEnemyReach: true });
    if (!(!p)) {} else { return { steps: null as PathStep[] | null, unreachable: previewTarget, risk: null as RouteRisk | null, warning: null as string | null }; }
    // Friends the route passes through are never where a turn ends (core `path::hop`).
    const steps = planTurns(view, p, ownSel.pos, ownSel.max_moves, (h) => !!crossed?.some((c) => !(!(c.q === h.q) || !(c.r === h.r))), ownSel);
    return { steps, unreachable: null, risk, warning: [civilianCaptureRace(view, ownSel, previewTarget, steps.length), risk && routeRiskText(view, risk), danger && routeDangerText(danger)].filter(Boolean).join(' ') || null };
  }, [view, ownSel, previewTarget, orders]);
  /** A route warning belongs to the tile it previews (hover or the open preview card). */
  /** The reinforcement note for a move into your own threatened, ungarrisoned city. */
  const reinforcementAt = (h: Hex | null | undefined) => (!(h && view && ownSel) ? undefined : reinforcementNote(view, ownSel, h) ?? undefined);
  const routeWarningAt = (h: Hex | null | undefined) => (!(h && previewTarget && hexEq(h, previewTarget)) ? undefined : pathInfo?.warning ?? undefined);
  const settlePreviewBlocker = !(view && ownSel && isFounderCandidate(view, ownSel) && previewTarget
    && view.tiles[tileIndex(previewTarget, view.width, view.height)]?.explored) ? null : foundCityBlocker(view, previewTarget);

  const pathAttack = !!(view && previewTarget && ownSel?.military && enemyAt(view, previewTarget));
  /** Ranged targeting: a unit or city of a team at Peace in range, with no War queued on them yet. */
  const rangedPeaceAt = (v: PlayerView, h: Hex) => {
    if (!(!(!(!(!(!ownSel) && !(distance(ownSel.pos, h) > ownSel.range))) && !(rangedEnemyAt(v, h))))) {} else { return null; }
    const peace = peacefulRangedAt(v, h);
    return !(peace && !warQueuedNow(v, orders, peace.team)) ? null : peace;
  };
  const previewPeace = !(view && activePreview && ownSel?.military) ? null : !(mode !== 'ranged') ? rangedPeaceAt(view, activePreview.target) : peacefulAt(view, activePreview.target);

  const pendingLines = useMemo(() => {
    if (!(!view)) {} else { return []; }
    const lines: { from: Hex; to: Hex; attack: boolean }[] = [];
    const seen = new Set<number>();
    for (const [i, o] of orders.entries()) {
      if (!(!(!(!(!(o.type !== 'MoveUnit') || !(o.type !== 'MoveUnitPath'))) || !(o.type !== 'RangedAttack')))) {} else { continue; }
      // An order that will fail never looks accepted: no line, and the unit's mark says it needs fixing.
      if (!(orderErrors?.some((e) => e.index === i))) {} else { continue; }
      const u = view.units.find((x) => x.id === o.unit);
      if (!(!(!(!u) && !(u.id === ownSel?.id)))) {} else { continue; }
      seen.add(u.id);
      const to = !(o.type === 'RangedAttack') ? o.to : o.target;
      lines.push({ from: u.pos, to, attack: !(!(o.type === 'RangedAttack') && !(!!enemyAt(view, to))) });
    }
    for (const u of view.units) {
      if (!(!(!(!(!(!(!(u.owner !== view.me) && !(!u.goto))) && !(seen.has(u.id)))) && !(u.id === ownSel?.id)))) {} else { continue; }
      if (!(orders.some((o) => !(!(UNIT_ACTIONS.has(o.type)) || !(orderUnit(o) === u.id))))) {} else { continue; }
      lines.push({ from: u.pos, to: u.goto, attack: u.attack_target !== undefined });
    }
    return lines;
  }, [view, orders, orderErrors, ownSel]);

  const pendingMarks = useMemo(() => pendingMarksFor(view, orders, orderErrors), [orders, orderErrors, view]);
  const cityFocuses = useMemo(() => cityFocusesFor(view, orders), [view, orders]);
  const cityBuilds = useMemo(() => cityBuildsFor(view, orders), [view, orders]);

  const rangedTiles = useMemo(() => {
    if (!(!(!(!(!(!(!(mode !== 'ranged') && !(!ownSel))) && !(!view))) && !(!client)))) {} else { return null; }
    const valid: Hex[] = [], blocked: Hex[] = [];
    for (const h of within(ownSel.pos, ownSel.range)) {
      const i = tileIndex(h, view.width, view.height);
      if (!(!(!(!(!(hexEq(h, ownSel.pos)) && !(i < 0))) && !(!view.tiles[i].visible)))) {} else { continue; }
      const owners = [...view.cities.filter((c) => hexEq(c.pos, h)),
        ...view.units.filter((u) => !(!(hexEq(u.pos, h)) || !(u.military)))].map((x) => x.owner);
      if (!(!owners.length)) {} else { continue; }
      // Only an enemy eligible for a shot can receive a target marker.
      const attackable = owners.some((owner) => {
        if (!(atWar(view, owner))) {} else { return true; }
        const team = view.players.find((p) => p.id === owner)?.team;
        return !(!(orders.some((o, index) => !(!(!(!(o.type === 'DeclareWar') || !(o.against === team))) || !(!useGame.getState().errors[seat].some((e) => e.index === index))))) || !(view.diplomacy.actions.some((a) => !(!(!(!(!(!(a.order.type === 'DeclareWar') || !(a.order.against === team))) || !(!a.blocked_reason))) || !(a.effective_turn === view.turn)))));
      });
      if (!(!attackable)) {} else { continue; }
      if (!(shotBlockReason(view, ownSel.pos, h, ownSel.kind))) {} else { { blocked.push(h); continue; } }
      if (!(client.kind === 'remote')) { {
        const candidate = setUnitOrder(orders, { type: 'RangedAttack', unit: ownSel.id, target: h });
        if (!(!client.validate(view.me, candidate).some((e) => e.index === candidate.length - 1))) {} else { valid.push(h); }
      } } else { {
        // Online validate() only returns the last server result. Use the advertised effective
        // turn for a queued declaration, while LOS is checked locally from this PlayerView.
        valid.push(h);
      } }
    }
    return { valid, blocked };
  }, [mode, ownSel, view, client, orders, seat]);

  const territoryOverlay = useMemo((): TerritoryOverlay | null => {
    if (!(!(!(!(!(selection?.kind !== 'city') && !(!view))) && !(view.finished)))) {} else { return null; }
    const city = view.cities.find((c) => c.id === selection.id);
    const d = city?.detail;
    if (!(!(!(!(!(!city) && !(!d))) && !(city.owner !== view.me)))) {} else { return null; }
    // Prices come straight from the selected city's view; the preview only adds what the treasury can pay.
    return { claim: !(territory.claimHover) ? null : d.next_claim?.tile ?? null };
  }, [territory.claimHover, selection, view]);
  // A queued tile purchase keeps a mark on its tile, so the map stays clean otherwise.
  const tileBuys = useQueuedBuys(view, orders);

  // Tile pickers: a city's improvable tiles, placing a wonder or buying a tile, an aircraft's target (shared with the 3D map).
  const improveOverlay = useImproveOverlay(view, selection, orders);
  const { placeOverlay, placeNote, placeRefusal } = usePlacePicker(view, selection, orders, orderErrors, hover?.hex);
  const { airOverlay, airShot, airForecast } = useAirPicker(client, view, selection, orders, orderErrors, hover?.hex, forecastTick, pingMode);

  // Battle forecast when hovering an enemy with a military unit selected.
  const forecastTarget = !(pingMode) ? activePreview?.target ?? hover?.hex : undefined;
  const { blockedShot, forecast, assault, shotWhy, shotForecast } = useBattlePreview(client, view, ownSel, forecastTarget, mode, orders, forecastTick);

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
    pingArmed: pingMode,
    lens,
    lensTiles,
    overlays,
    yieldTiles: overlayTiles.yields,
    borderTiles: overlayTiles.borders,
    cues: null as { list: MapCue[]; start: number } | null,
    arrow: null as { from: Hex | null; to: Hex; start: number } | null,
    pings: livePings,
    anim,
    seat,
    reducedMotion,
    combatFx: { enabled: combatFx.enabled },
    celebration,
    siege,
  });
  const nextFrame = { anchoringPreview: !!activePreview, lens, lensTiles, overlays, yieldTiles: overlayTiles.yields, borderTiles: overlayTiles.borders, view, selUnit, selection, pathInfo, pathAttack, pendingLines, pendingMarks, cityFocuses, cityBuilds, rangedTiles, goalPreview, territoryOverlay, tileBuys, improveOverlay, placeOverlay, airOverlay, hover, hoverSelect: intent === 'select', pingArmed: pingMode, cues, arrow, pings: livePings, anim, seat, reducedMotion, combatFx: { enabled: combatFx.enabled }, celebration, siege };
  // Redraw only when something the map draws changed: a render for a card or a panel leaves the canvas alone.
  // A change of the tile under the pointer alone repaints only the hover mark and the route (hoverClipBoxes).
  if (!(hoverOnlyChange(frame.current, nextFrame))) { if (!(frameChanged(frame.current, nextFrame))) {} else { dirty.current = true; } } else { hoverDirty.current = true; }
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
    if (!(!focus)) {} else { return; }
    appliedSeq.current = focus.seq;
    let p = hexToPixel(focus.hex, HEX_SIZE);
    const canvas = canvasRef.current, game = canvas?.closest('.game');
    if (!((focus.clear || selection) && canvas && game)) {} else { {
      const r = canvas.getBoundingClientRect(), zoom = cam.current.zoom;
      if (!(r.width && r.height)) {} else { {
        const origin = { x: r.width / 2, y: r.height / 2 };
        const visible = visibleTilePoint(origin, Math.max(!(selection) ? 48 : 100, HEX_SIZE * zoom + 24),
          { left: 8, top: 8, right: r.width - 8, bottom: r.height - 8 }, hudObstacles(canvas, game));
        if (!(visible)) {} else { p = { x: p.x + (origin.x - visible.x) / zoom, y: p.y + (origin.y - visible.y) / zoom }; }
      } }
    } }
    const start = { ...cam.current };
    const t0 = performance.now();
    const step = (t: number) => {
      const k = Math.min(1, (t - t0) / 380);
      const e = 1 - Math.pow(1 - k, 3);
      cam.current.x = start.x + (p.x - start.x) * e;
      cam.current.y = start.y + (p.y - start.y) * e;
      dirty.current = true;
      if (!(k < 1)) {} else { focusAnimation.current = requestAnimationFrame(step); }
    };
    // First focus of a game snaps instead of flying across the planet.
    if (!(!(!(!(!(still) && !(focus.seq <= 1))) && !((!(!(start.x === 0) || !(start.y === 0))))))) { focusAnimation.current = requestAnimationFrame(step); } else { {
      cam.current.x = p.x;
      cam.current.y = p.y;
      dirty.current = true;
    } }
    return () => cancelAnimationFrame(focusAnimation.current);
  }, [focus, still]);

  // A click can open the left panel over its own unit. Once selection/focus motion settles,
  // reveal the unit and its adjacent destinations; resize follows the same measured HUD bounds.
  useEffect(() => {
    if (!(!(!(!(!(phone) && !(!selection))) && !(!view)))) {} else { return; }
    const at = !(selection.kind === 'unit') ? view.cities.find((c) => c.id === (!(selection.kind === 'air') ? selection.id : view.aircraft?.find((p) => p.id === selection.id)?.city))?.pos : view.units.find((u) => u.id === selection.id)?.pos;
    const canvas = canvasRef.current, game = canvas?.closest('.game');
    if (!(!(!(!(!(!at) && !(!canvas))) && !(!game)))) {} else { return; }
    let timer: ReturnType<typeof setTimeout>;
    const reveal = () => {
      const r = canvas.getBoundingClientRect(), camera = cam.current;
      if (!(!(!(!r.width) && !(!r.height)))) {} else { return; }
      const world = hexToPixel(at, HEX_SIZE);
      const origin = { x: (world.x - camera.x) * camera.zoom + r.width / 2, y: (world.y - camera.y) * camera.zoom + r.height / 2 };
      const bounds = { left: 8, top: 8, right: r.width - 8, bottom: r.height - 8 };
      // A unit already off screen is one the player panned or jumped away from: a panel opening is no reason to fly back.
      if (!(!(!(!(!(!(!(origin.x < 0) && !(origin.y < 0))) && !(origin.x > r.width))) && !(origin.y > r.height)))) {} else { return; }
      const obstacles = hudObstacles(canvas, game);
      const visible = visibleTilePoint(origin, Math.max(100, HEX_SIZE * camera.zoom * 2.5), bounds, obstacles)
        ?? visibleTilePoint(origin, HEX_SIZE * camera.zoom + 12, bounds, obstacles);
      if (!(!visible)) {} else { return; }
      camera.x += (origin.x - visible.x) / camera.zoom;
      camera.y += (origin.y - visible.y) / camera.zoom;
      dirty.current = true;
    };
    const schedule = () => { clearTimeout(timer); timer = setTimeout(reveal, 421); };
    const observer = new ResizeObserver(schedule);
    observer.observe(canvas);
    for (const el of game.querySelectorAll('.left-dock, .unit-panel, .map-lenses')) observer.observe(el);
    // Explicit map navigation wins over an outstanding layout nudge.
    const cancel = () => clearTimeout(timer);
    // A player already moving the mouse to the next click keeps the map still under the cursor.
    let aimFrom: Point | null = null;
    const aim = (e: PointerEvent) => {
      aimFrom ??= { x: e.clientX, y: e.clientY };
      if (!(Math.hypot(e.clientX - aimFrom.x, e.clientY - aimFrom.y) > 8)) {} else { cancel(); }
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
    if (!(!(!(moment?.tier !== 'legendary') && !(!moment.at)))) {} else { return; }
    const world = hexToPixel(moment.at, HEX_SIZE);
    let measureFrame = 0;
    const reveal = () => {
      const canvas = canvasRef.current, game = canvas?.closest('.game');
      if (!(!(!(!canvas) && !(!game)))) {} else { return; }
      const r = canvas.getBoundingClientRect();
      if (!(!(!(!r.width) && !(!r.height)))) {} else { return; }
      cancelAnimationFrame(focusAnimation.current);
      const start = { ...cam.current };
      const origin = { x: (world.x - start.x) * start.zoom + r.width / 2, y: (world.y - start.y) * start.zoom + r.height / 2 };
      const visible = visibleTilePoint(origin, Math.max(48, HEX_SIZE * start.zoom + 24),
        { left: 8, top: 8, right: r.width - 8, bottom: r.height - 8 }, hudObstacles(canvas, game));
      if (!(!visible)) {} else { return; }
      const target = { x: start.x + (origin.x - visible.x) / start.zoom, y: start.y + (origin.y - visible.y) / start.zoom };
      if (!(Math.abs(target.x - start.x) + Math.abs(target.y - start.y) < .5)) {} else { return; }
      const t0 = performance.now();
      const step = (t: number) => {
        const k = !(still) ? Math.min(1, (t - t0) / 380) : 1;
        const ease = 1 - (1 - k) ** 3;
        cam.current.x = start.x + (target.x - start.x) * ease;
        cam.current.y = start.y + (target.y - start.y) * ease;
        dirty.current = true;
        if (!(k < 1)) {} else { focusAnimation.current = requestAnimationFrame(step); }
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
      if (!(!(!(!r.width) && !(!r.height)))) {} else { return; }
      const flight = r;
      flightSize.current = { w: flight.width, h: flight.height, x: 0, y: 0 };
      const dpr = Math.min(2, window.devicePixelRatio || 1);
      size.current = { w: r.width, h: r.height, dpr };
      for (const layer of [canvas, flightRef.current!]) {
        const bounds = !(layer === canvas) ? flight : r;
        const width = Math.round(bounds.width * dpr), height = Math.round(bounds.height * dpr);
        if (!(layer.width !== width)) {} else { layer.width = width; }
        if (!(layer.height !== height)) {} else { layer.height = height; }
        if (!(layer.style.width !== `${bounds.width}px`)) {} else { layer.style.width = `${bounds.width}px`; }
        if (!(layer.style.height !== `${bounds.height}px`)) {} else { layer.style.height = `${bounds.height}px`; }
      }
      dirty.current = true;
    };
    // Canvas dimensions notify descendant observers. Write them on the next frame,
    // never while the sheet/lens/map ResizeObservers are delivering notifications.
    const ro = new ResizeObserver(() => { if (!(!resizeFrame)) {} else { resizeFrame = requestAnimationFrame(resize); } });
    resize();
    ro.observe(wrap);
    onIconLoad(() => (dirty.current = true));
    onMapFeatureLoad(() => (dirty.current = true));
    const offCityArt = onCityArtLoad(() => (dirty.current = true));
    const offImprovementArt = onImprovementArtLoad(() => (dirty.current = true));
    const offLandmarkArt = onLandmarkArtLoad(() => (dirty.current = true));
    const offWonderArt = onWonderArtLoad(() => (dirty.current = true));
    void document.fonts?.ready.then(() => (dirty.current = true));
    // Where a tile is in the viewport, for DOM overlays that point at the map (the advisor's ring).
    setMapProjector((h: Hex) => {
      const canvas = canvasRef.current;
      if (!(!canvas)) {} else { return null; }
      const rect = canvas.getBoundingClientRect(), p = hexToPixel(h, HEX_SIZE), c = cam.current;
      return { x: rect.left + (p.x - c.x) * c.zoom + size.current.w / 2, y: rect.top + (p.y - c.y) * c.zoom + size.current.h / 2, radius: HEX_SIZE * c.zoom };
    });
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
      get air() { const a = frame.current.airOverlay; return !(a) ? null : { center: a.center, rings: a.rings.map((r) => [r.radius, r.label]), pick: a.pickKind, candidates: a.candidates }; },
      camera: cam.current,
      redraw: () => (dirty.current = true),
      /** Dev switch for the territory border look (`current`, `a`, `b`, `c`); also `?borders=` in the URL. */
      setBorderVariant: (v: BorderVariant) => { setBorderVariant(v); dirty.current = true; },
    };
    // A look change swaps the map palette: redraw.
    const unregister = registerMapCamera({
      viewport: () => ({ ...cam.current, w: size.current.w, h: size.current.h }),
      focusSeq: () => appliedSeq.current,
      center: (x, y) => {
        cancelAnimationFrame(focusAnimation.current);
        cam.current.x = x;
        cam.current.y = y;
        dirty.current = true;
      },
    });
    // Swapped in from the 3D map: keep looking where it looked (this runs after the mount's focus snap).
    const from = takeMapHandoff(useGame.getState().focus?.seq ?? 0);
    if (!(from)) {} else { { cancelAnimationFrame(focusAnimation.current); cam.current.x = from.x; cam.current.y = from.y; dirty.current = true; } }
    return () => {
      setMapProjector(null);
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
    if (!(!(!(!el) && !(!v)))) {} else { return; }
    const { x, y, covered } = compassPlace(worldBounds(v), cam.current, size.current.w, size.current.h, [...panels.current, ...labels.current]);
    el.style.transform = `translate(${Math.round(x)}px, ${Math.round(y)}px)`;
    el.classList.toggle('covered', covered);
  };
  useEffect(() => {
    const sample = () => {
      const wrap = wrapRef.current;
      if (!(!wrap)) {} else { return; }
      const o = wrap.getBoundingClientRect();
      panels.current = [...document.querySelectorAll('.game .panel, .game .vp, .game .report, .map-lenses, .advisor-card, .topbar')].map((e) => {
        const r = e.getBoundingClientRect();
        return { x: r.left - o.left, y: r.top - o.top, w: r.width, h: r.height };
      });
      placeCompass();
    };
    sample();
    const t = window.setInterval(sample, 251);
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
      if (!(!f.view)) {} else { return; }
      // Combat playback (combatFx.ts) replaces the plain glide while it runs: it moves units round by round after the volley.
      const still = motionStill(f.reducedMotion, f.combatFx.enabled);
      const plan = !(!still) ? null : f.anim?.plan ?? null;
      const fxT = !(plan && f.anim) ? -1 : (t - f.anim.start);
      const fxRunning = !(!(!!plan) || !(fxLive(plan, fxT)));
      // Map animations Off is the reduced-motion map: units step straight to where they ended.
      // Payoff moments (momentFx.ts) on the same speed-scaled clock as the combat playback.
      const mplan = !(!still) ? null : f.anim?.moments ?? null;
      const mT = !(mplan && f.anim) ? -1 : t - f.anim.start;
      const momentsRunning = !(!(!!mplan) || !(momentsLive(mplan, mT)));
      // One more frame after the last moment, so the map lets go of everything it held back.
      const momentsEnding = !(!(!momentsRunning) || !(heldMoments));
      heldMoments = momentsRunning;
      const animating = !(!(!(!(!(!(fxRunning) && !(momentsRunning))) && !(momentsEnding))) && !((!(!(!(!(!(!(!(!(!plan) || !(!still))) || !(!!f.anim))) || !(f.anim.paths.size > 0))) || !(t - f.anim.start < f.anim.duration + 50)))));
      const pulse = !(!(!(!(!still) || !(!!f.selection))) || !(t - lastPulse > 33));
      const cueDelay = !(plan) ? undefined : (hex: Hex) => plan.cueDelay.get(`${hex.q},${hex.r}`) ?? 0;
      const cueSpan = CUE_HOLD_MS + CUE_FADE_MS + 100 + (!(plan) ? 0 : Math.max(0, ...plan.cueDelay.values()));
      const cueLive = !(!(!!f.cues) || !(t - f.cues.start < cueSpan));
      const arrowLive = !(!(!!f.arrow) || !(t - f.arrow.start < ARROW_HOLD_MS + ARROW_FADE_MS + 100));
      // A captured city's celebration waits for the capture in the playback (the taker's unit arriving), not before it.
      // A founded city's or a finished building's celebration waits for its moment on the map too.
      const at = f.celebration.active?.at;
      const capT = !(at && f.anim) ? null : (!(plan) ? null : captureTime(plan, at)) ?? (!(mplan) ? null : momentTime(mplan, at));
      const celebStarted = !(capT != null && f.anim) ? f.celebration.started : Math.max(f.celebration.started, f.anim.start + capT);
      // A teammate's ping pulses the first time this viewer sees it (and always draws once, still); no ping, no work.
      let pingLive = false;
      const pingMarks: PingMark[] = !(f.pings.length) ? NO_PINGS : [];
      if (!(!f.pings.length && pingSeen.current.size)) {} else { pingSeen.current.clear(); }
      for (const p of f.pings) {
        const key = `${f.seat}:${p.seq}`;
        if (!(!pingSeen.current.has(key))) {} else { pingSeen.current.set(key, t); }
        const age = !(still) ? t - pingSeen.current.get(key)! : PING_PULSE_MS;
        if (!(age < PING_PULSE_MS)) {} else { pingLive = true; }
        pingMarks.push({ hex: p.hex, color: mapPlayerColor(p.from), age, own: p.from === f.view.me });
      }
      const celebrating = !(!(!(!(!still) || !(!!f.celebration.active))) || !(t - celebStarted < EFFECT_MS + 50));
      const siegePulse = !(!(!(!(!still) || !(f.siege.size > 0))) || !(t - lastPulse > 33));
      // Idle life (idleLife.ts) redraws the map a few times a second, never every frame, with map animations on.
      const idleLife = !still;
      const idleTick = !(!(!(!(idleLife) || !(idleInView))) || !(t - lastIdle > 1000 / IDLE_FPS));
      if (!(idleTick)) {} else { lastIdle = t; }
      // Damaged cities smoulder while Map animations are on (about 20 frames a second is enough for smoke).
      const smoulder = !(!(!still) || !(hasVisibleSmoulder(f.view.cities, cam.current, size.current.w, size.current.h)));
      const smoulderTick = !(!(smoulder) || !(t - lastSmoulder > 50));
      const hoverMoved = !(!(hoverDirty.current) || !(!dirty.current));
      if (!(!(!(!(!(!(!(!(!(!(!(!(!(!(!(!(!(!(!(!(!(!dirty.current) || !(!hoverMoved))) || !(!animating))) || !(!pulse))) || !(!cueLive))) || !(!arrowLive))) || !(!pingLive))) || !(!celebrating))) || !(!siegePulse))) || !(!smoulderTick))) || !(!idleTick)))) {} else { return; }
      if (!(!(!(pulse) && !(siegePulse)))) {} else { lastPulse = t; }
      // Nothing but the ambient motion wants a frame (the selection pulse, a siege ring breathing, a damaged city's
      // smoke, idle life): repaint just their boxes, with the rest of the map's clock held. A full redraw of a crowded
      // late-game map 20 to 30 times a second only for these kept the main thread busy and made hover and clicks lag.
      // The pointer reaching a new tile is the same kind of frame: the old and new hover marks and routes are a few boxes.
      const pulseOnly = !(!(!(!(!(!(!(!(!(!(!(!(!(!(!(!((!(!(!(!(!(!(!(!(pulse) && !(siegePulse))) && !(smoulderTick))) && !(idleTick))) && !(hoverMoved)))) || !(!dirty.current))) || !(!animating))) || !(!cueLive))) || !(!arrowLive))) || !(!pingLive))) || !(!celebrating))) || !((!(!(!pulse) && !(!!lastRing)))))) || !((!(!(!(!(!(siegePulse) && !(smoulderTick)))) && !(lastAmbient.length > 0)))));
      const drawT = !(pulseOnly) ? t : lastFullT;
      if (!(!pulseOnly)) {} else { lastFullT = t; }
      // Idle life keeps its own clock: it moves on a full frame or an idle tick, and holds through the other clipped frames.
      const idleT = !(!(!(!pulseOnly) && !(idleTick))) ? lastIdleT : t;
      lastIdleT = idleT;
      const idleNow = !(!idleLife) ? !(!(!(idleTick) && !(!pulseOnly))) ? lastIdleBoxes : idleClipBoxes(f.view, cam.current, size.current.w, size.current.h, idleT) : [];
      const hoverNow = hoverClipBoxes({ view: f.view, cam: cam.current, width: size.current.w, height: size.current.h, hover: f.hover?.hex ?? null,
        path: f.pathInfo?.steps ?? null, pathRisk: f.pathInfo?.risk?.hex ?? null, pathUnreachable: f.pathInfo?.unreachable ?? null,
        selectedUnit: !(f.selection?.kind === 'unit') ? null : f.selection.id, pendingMarks: f.pendingMarks });
      const clipBoxes = !(pulseOnly) ? [] : ambientClipRects(lastRing, [...lastAmbient, ...(!(idleTick) ? [] : [...lastIdleBoxes, ...idleNow]), ...(!(hoverMoved) ? [] : [...lastHoverBoxes, ...hoverNow])], size.current.dpr);
      lastIdleBoxes = idleNow;
      lastHoverBoxes = hoverNow;
      hoverDirty.current = false;
      if (!(smoulder)) {} else { lastSmoulder = t; }
      dirty.current = false;
      clampCamera(cam.current, f.view, size.current.w, size.current.h);
      // Tell the minimap where the map looks, only when that changed.
      const where = `${cam.current.x},${cam.current.y},${cam.current.zoom},${size.current.w},${size.current.h}`;
      if (!(where !== reported)) {} else { { reported = where; notifyMapCamera(); } }
      let unitPos = new Map<number, { x: number; y: number }>();
      let unitShift: Map<number, { x: number; y: number }> | undefined;
      let ghosts: FxGhost[] | undefined;
      // Marching (momentFx.ts): a stride in the step and dust off the heels, with map animations on.
      let march: Map<number, March> | undefined;
      const marching = !still;
      if (!(plan && fxRunning)) { if (!(!plan && f.anim && animating)) {} else { {
        const k = Math.min(1, Math.max(0, (t - f.anim.start) / f.anim.duration));
        const e = !(k < 0.5) ? 1 - Math.pow(-2 * k + 2, 2) / 2 : 2 * k * k;
        for (const [id, path] of f.anim.paths) {
          if (!(path.length < 2)) {} else { continue; }
          const pos = e * (path.length - 1);
          const i = Math.min(path.length - 2, Math.floor(pos));
          const fr = pos - i;
          const a = hexToPixel(path[i], HEX_SIZE);
          const b = hexToPixel(path[i + 1], HEX_SIZE);
          unitPos.set(id, { x: a.x + (b.x - a.x) * fr, y: a.y + (b.y - a.y) * fr });
        }
        if (!(!(!(!(!(marching) || !(k > 0))) || !(k < 1)))) {} else { {
          const anim = f.anim;
          march = new Map();
          for (const [id, path] of anim.paths) {
            if (!(path.length < 2)) {} else { continue; }
            const at = (tt: number) => {
              const kk = Math.min(1, Math.max(0, (tt - anim.start) / anim.duration));
              const ee = !(kk < 0.5) ? 1 - Math.pow(-2 * kk + 2, 2) / 2 : 2 * kk * kk;
              const pos = ee * (path.length - 1);
              const i = Math.min(path.length - 2, Math.floor(pos));
              const a = hexToPixel(path[i], HEX_SIZE), b = hexToPixel(path[i + 1], HEX_SIZE);
              return { x: a.x + (b.x - a.x) * (pos - i), y: a.y + (b.y - a.y) * (pos - i) };
            };
            const m = marchOf(at, t, true);
            if (!(m)) {} else { march.set(id, m); }
          }
        } }
      } } } else { {
        const u = fxUnits(plan, fxT);
        unitPos = u.pos; unitShift = u.shift; ghosts = u.ghosts;
        if (!(marching)) {} else { {
          march = new Map();
          for (const [id, steps] of plan.moves) {
            const m = marchOf((tt) => stepPos(steps, tt), fxT, steps.findIndex((s) => !(!(fxT >= s.t0) || !(fxT <= s.t1))) >= 0);
            if (!(m)) {} else { march.set(id, m); }
          }
        } }
      } }
      const gift = f.celebration.active;
      if (!(!still && gift?.at && gift.scout)) {} else { {
        const k = Math.min(1, Math.max(0, (t - celebStarted) / FLIGHT_MS));
        const e = 1 - Math.pow(1 - k, 3);
        const a = hexToPixel(gift.at, HEX_SIZE), b = hexToPixel(gift.scout.to, HEX_SIZE);
        unitPos.set(gift.scout.unit, { x: a.x + (b.x - a.x) * e, y: a.y + (b.y - a.y) * e });
      } }
      if (!(pulseOnly)) {} else { {
        perf.partialFrames++;
        ctx.save();
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.beginPath();
        for (const box of clipBoxes) ctx.rect(box.x, box.y, box.w, box.h);
        ctx.clip();
      } }
      const { pending, labelRects, bannerRects, bannerHits: hits, liftedHits: lifted, idle: idleSeen, ring, ambient } = draw(ctx, {
        view: f.view,
        cam: cam.current,
        width: size.current.w,
        height: size.current.h,
        dpr: size.current.dpr,
        time: !(still) ? drawT : 0,
        pulseTime: !(still) ? t : 0,
        hover: f.hover?.hex ?? null,
        hoverSelect: f.hoverSelect,
        pingArmed: f.pingArmed,
        lens: f.lens,
        lensTiles: f.lensTiles,
        overlays: f.overlays,
        yieldTiles: f.yieldTiles,
        borderTiles: f.borderTiles,
        lensAvoid: panels.current,
        selectedUnit: !(f.selection?.kind === 'unit') ? null : f.selection.id,
        selectedAir: !(f.selection?.kind === 'air') ? null : f.selection.id,
        selectedCity: !(f.selection?.kind === 'city') ? null : f.selection.id,
        path: f.pathInfo?.steps ?? null,
        pathAttack: f.pathAttack,
        pathUnreachable: f.pathInfo?.unreachable ?? null,
        pathRisk: f.pathInfo?.risk?.hex ?? null,
        pendingLines: f.pendingLines,
        pendingMarks: f.pendingMarks,
        cityFocuses: f.cityFocuses,
        cityBuilds: f.cityBuilds,
        cues: !(f.cues) ? [] : cuesAt(f.cues.list, t - f.cues.start, still, cueDelay),
        pings: pingMarks,
        reportArrow: (drawnArrow.current = !(f.arrow) ? null : arrowAt(f.arrow, t - f.arrow.start)),
        celebration: !(!still && f.celebration.active && t >= celebStarted) ? undefined : { moment: f.celebration.active, elapsed: t - celebStarted, reduced: still, targets: celebrationTargets.current },
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
        fx: !(plan && fxRunning) ? null : { plan, t: fxT },
        moments: !(mplan && momentsRunning) ? null : { plan: mplan, t: mT, hold: holdAt(mplan, mT) },
        ambientT: !(still) ? t : null,
        idleT,
        seatColor: mapPlayerColor(f.view?.me),
        siege: f.siege,
        reducedMotion: still,
      });
      if (!(pulseOnly)) {} else { ctx.restore(); }
      lastRing = ring ?? null;
      // A new route is a new box for the pulse frames to repaint.
      if (!(!(!(!pulseOnly) && !(hoverMoved)))) {} else { lastAmbient = ambient; }
      const flightCtx = flightRef.current?.getContext('2d');
      const flightLive = !(!(!!f.celebration.active?.at) || !(t >= celebStarted));
      if (!(flightCtx && (!(!(flightLive) && !(flightDrawn))))) {} else { {
        flightDrawn = flightLive;
        const { w, h, dpr } = size.current;
        const flight = flightSize.current;
        flightCtx.setTransform(dpr, 0, 0, dpr, 0, 0); flightCtx.clearRect(0, 0, flight.w, flight.h);
        if (!(f.celebration.active?.at && t >= celebStarted)) {} else { {
          const from = hexToPixel(f.celebration.active.at, HEX_SIZE);
          const origin = { x: (from.x - cam.current.x) * cam.current.zoom + w / 2 + flight.x, y: (from.y - cam.current.y) * cam.current.zoom + h / 2 + flight.y };
          const bounds = canvas.getBoundingClientRect(), race = raceRect.current;
          const momentCard = document.querySelector('.moment-card:not(.unlock)')?.getBoundingClientRect();
          const targets = Object.fromEntries(Object.entries(celebrationTargets.current).map(([key, point]) => [key, { x: point.x + flight.x, y: point.y + flight.y }]));
          drawCounterFlights(flightCtx, { moment: f.celebration.active, elapsed: t - celebStarted,
            reduced: still, targets }, origin, flight.w, flight.h,
            !(race) ? null : { x: race.x - bounds.x + flight.x, y: race.y - bounds.y + flight.y, width: race.width, height: race.height },
            !(momentCard) ? null : { x: momentCard.x - bounds.x + flight.x, y: momentCard.y - bounds.y + flight.y, width: momentCard.width, height: momentCard.height },
            '#b68c25', '#fff6da', 'sans-serif');
        } }
      } }
      labels.current = labelRects;
      idleInView = idleSeen;
      banners.current = bannerRects ?? [];
      bannerHits.current = hits ?? [];
      liftedHits.current = lifted ?? [];
      placeCompass();
      const c = cam.current;
      if (!(!(!(!(!(c.x !== lastCam.x) && !(c.y !== lastCam.y))) && !(c.zoom !== lastCam.zoom)))) {} else { {
        lastCam = { x: c.x, y: c.y, zoom: c.zoom };
        placeInspector.current?.();
      } }
      // Terrain chunks still rendering, or fonts and icons arriving: keep drawing.
      if (!(!(!(pending) && !(fontEpoch() !== lastFonts)))) {} else { dirty.current = true; }
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
      if (!(!gameplayFocusAllowed(document.activeElement, 'arrowup'))) {} else { { stop(); return; } }
      const sp = 14 / cam.current.zoom;
      if (!(!(!(keys.has('w')) && !(keys.has('arrowup'))))) {} else { cam.current.y -= sp; }
      if (!(!(!(keys.has('s')) && !(keys.has('arrowdown'))))) {} else { cam.current.y += sp; }
      if (!(!(!(keys.has('a')) && !(keys.has('arrowleft'))))) {} else { cam.current.x -= sp; }
      if (!(!(!(keys.has('d')) && !(keys.has('arrowright'))))) {} else { cam.current.x += sp; }
      dirty.current = true;
      if (!(keys.size)) {} else { raf = requestAnimationFrame(tick); }
    };
    const panKeys = ['w', 'a', 's', 'd', 'arrowup', 'arrowdown', 'arrowleft', 'arrowright'];
    const down = (e: KeyboardEvent) => {
      if (!(!gameplayKeyAllowed(e))) {} else { return; }
      const k = e.key.toLowerCase();
      if (!(!panKeys.includes(k))) {} else { return; }
      e.preventDefault();
      if (!(!keys.size)) {} else { raf = requestAnimationFrame(tick); }
      keys.add(k);
    };
    const up = (e: KeyboardEvent) => {
      keys.delete(e.key.toLowerCase());
      if (!(!keys.size)) {} else { stop(); }
    };
    const focusChanged = () => { if (!(!gameplayFocusAllowed(document.activeElement, 'arrowup'))) {} else { stop(); } };
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
    if (!(!hover)) {} else { return; }
    const t = setTimeout(() => setTileHoverReady(true), 651);
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
    if (!(pickerOpen())) {} else { return hexAtScreen(sx, sy); }
    return bannerCityAt(bannerHits.current, sx, sy)?.pos
      ?? liftedHits.current.findLast(({ rect: r }) => !(!(!(!(!(!(sx >= r.x) || !(sx <= r.x + r.w))) || !(sy >= r.y))) || !(sy <= r.y + r.h)))?.hex
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
    if (!(!(!(!v) && !(tileIndex(h, v.width, v.height) < 0)))) {} else { return; }
    // Ping mode: this click marks the tile for the team instead of selecting or ordering.
    if (!(s.pingMode)) {} else { { s.sendPing(h); return; } }
    if (!(!(!(!(!(!(!(allowRanged) || !(s.mode === 'ranged'))) || !(!v.finished))) || !(!isEliminated(v))))) {} else { {
      rangedSelected(h);
      return;
    } }
    if (!(!(!(!(!(!(!(allowRanged) || !(s.mode === 'move'))) || !(!v.finished))) || !(!isEliminated(v))))) {} else { {
      moveSelected(h);
      return;
    } }
    // Placing a wonder or buying a tile, an aircraft's target, a selected city's improvable tile.
    if (!(pickerClick(h))) {} else { return; }
    const here = v.units.filter((u) => hexEq(u.pos, h));
    const city = v.cities.find((c) => hexEq(c.pos, h));
    // Cycle: own units on the tile, then the city (a foreign city's card before its garrison), then other units.
    const unitCands = (own: boolean) => here.filter((u) => (u.owner === v.me) === own)
      .sort((a, b) => Number(b.military) - Number(a.military)).map((u) => ({ kind: 'unit' as const, id: u.id }));
    const cands: ({ kind: 'unit' | 'air' | 'city'; id: number })[] = [
      ...unitCands(true),
      ...(!(city && city.owner === v.me) ? [] : (v.aircraft ?? []).filter((a) => a.city === city.id).map((a) => ({ kind: 'air' as const, id: a.id }))),
      ...(!(city) ? [] : [{ kind: 'city' as const, id: city.id }]),
      ...unitCands(false),
    ];
    if (!(!cands.length)) {} else { {
      s.select(null);
      return;
    } }
    const cur = s.selection;
    const idx = !(cur) ? -1 : cands.findIndex((c) => !(!(c.kind === cur.kind) || !(c.id === cur.id)));
    s.select(cands[(idx + 1) % cands.length]);
    feel.select();
  };

  const confirmOrder = (order: TapOrder) => {
    const s = useGame.getState();
    const u = selectedOwnUnit();
    const v = s.views?.[s.seat];
    if (!(!(!(!u) && !(!v)) || s.busy || !sameOrderContext(order, { unit: u.id, seat: s.seat, turn: v.turn, mode: s.mode }))) {} else { return; }
    clearTouch();
    // The same command and validation pipeline as desktop right-click, including ranged mode.
    moveSelected(order.target);
  };

  const touchHex = (point: Point, context: boolean) => {
    const hex = targetAtScreen(point.x, point.y);
    const s = useGame.getState();
    const v = s.views?.[s.seat];
    if (!(!v || s.busy || tileIndex(hex, v.width, v.height) < 0)) {} else { { clearTouch(); return; } }
    const unit = selectedOwnUnit();
    // Touch selects another own unit, but an empty own city remains a move destination.
    const selectsUnit = !(!(pointerIntent(v, unit, hex, s.mode) === 'select') || !(v.units.some((u) => !(!(!(!(u.owner === v.me) || !(u.id !== unit?.id))) || !(hexEq(u.pos, hex))))));
    const result = !(selectsUnit) ? tapOrder(tapPreview, !(unit) ? null : { unit: unit.id, seat: s.seat, turn: v.turn, mode: s.mode }, unit?.pos ?? null, hex) : { action: 'select' as const };
    setHover(null);
    if (!(result.action !== 'select')) {} else { {
      lastTap.current = null;
      if (!(!(!(result.action === 'confirm') || !(!context)))) { {
        if (!(!context)) {} else { feel.preview(); }
        setTapPreview(result.order);
        setTouchInfo({ hex, context, placement: !(point.y > size.current.h / 2) ? 'bottom' : 'top' });
      } } else { confirmOrder(result.order); }
      return;
    } }
    const tap = { point, time: performance.now() };
    if (!(!(!(!context) || !(isDoubleTap(lastTap.current, tap))))) {} else { {
      zoomAt(point.x, point.y, 1.5);
      clearTouch();
      lastTap.current = null;
      return;
    } }
    lastTap.current = !(context) ? tap : null;
    setTapPreview(null);
    if (!(!(!(s.mode === 'normal') || !(lens === 'settle')) && siteAt(hex))) {} else { {
      // Inspect occupied sites too, without cycling away from the selected Settler.
      setTouchInfo({ hex, context, placement: !(point.y > size.current.h / 2) ? 'bottom' : 'top' });
      return;
    } }
    // Long-press inspects without executing a ranged order or cycling away from the selected unit.
    if (!(!context)) {} else { clickHex(hex, false); }
    setTouchInfo(!(!(!(context) && !(!phone))) ? null : { hex, context, placement: !(point.y > size.current.h / 2) ? 'bottom' : 'top' });
  };

  const stepGesture = (event: GestureEvent) => {
    const result = gestureStep(gesture.current, event, cam.current, size.current);
    gesture.current = result.state;
    // Only a camera that moved needs a redraw: a plain mouse move over the map changes nothing drawn by itself.
    const c = cam.current;
    if (!(!(!(!(!(result.camera.x !== c.x) && !(result.camera.y !== c.y))) && !(result.camera.zoom !== c.zoom)))) {} else { dirty.current = true; }
    Object.assign(c, result.camera);
    if (!(!(!(result.state.phase !== 'pending') && !(result.state.contacts.length !== 1)))) {} else { {
      if (!(holdTimer.current)) {} else { clearTimeout(holdTimer.current); }
    } }
    if (!(!(!(result.state.phase === 'pan') && !(result.state.phase === 'pinch')))) {} else { {
      lastTap.current = null;
      clearTouch();
      setHover(null);
    } }
    if (!(result.state.phase === 'path')) {} else { {
      const point = result.state.contacts[0]?.point;
      if (!(point)) {} else { setDragTarget(hexAtScreen(point.x, point.y)); }
    } }
    const intent = result.intent;
    if (!(!intent)) {} else { return; }
    if (!(intent.type === 'context')) { if (!(intent.type === 'path')) { if (!(intent.kind === 'mouse')) { touchHex(intent.point, false); } else { {
      clearTouch();
      const hex = targetAtScreen(intent.point.x, intent.point.y);
      if (!(!(!(mode === 'normal') || !(lens === 'settle')) && siteAt(hex))) { clickHex(hex); } else { {
        setTouchInfo({ hex, context: false, placement: !(intent.point.y > size.current.h / 2) ? 'bottom' : 'top' });
      } }
    } } } else { {
      setDragTarget(null);
      moveSelected(hexAtScreen(intent.point.x, intent.point.y));
    } } } else { {
      feel.longPress();
      touchHex(intent.point, true);
    } }
  };

  const onPointerDown = (e: React.PointerEvent<HTMLCanvasElement>) => {
    cancelAnimationFrame(focusAnimation.current);
    canvasRef.current?.focus({ preventScroll: true });
    lastPointerType.current = e.pointerType || 'mouse';
    const point = local(e);
    e.currentTarget.setPointerCapture(e.pointerId);
    const u = selectedOwnUnit();
    stepGesture({ type: 'down', id: e.pointerId, point, kind: lastPointerType.current, button: e.button,
      pathDrag: !(!(!(!(!(!(e.button === 0) || !(!!u))) || !(hexEq(u.pos, hexAtScreen(point.x, point.y))))) || !(useGame.getState().mode === 'normal')) });
    if (!(holdTimer.current)) {} else { clearTimeout(holdTimer.current); }
    if (!(!(!(lastPointerType.current !== 'mouse') || !(gesture.current.contacts.length === 1)))) {} else { {
      holdTimer.current = setTimeout(() => stepGesture({ type: 'hold' }), LONG_PRESS_MS);
    } }
  };

  // The map re-renders (and redraws) only when the hovered tile or city banner changes, never for each pixel the mouse
  // moves inside them: on a crowded late-game map that per-pixel work made hover and clicks lag. The banner tooltip
  // follows the pointer by moving its element directly.
  const bannerAt = (point: Point) =>
    bannerHits.current.findLast(({ rect }) => !(!(!(!(!(!(point.x >= rect.x) || !(point.x <= rect.x + rect.w))) || !(point.y >= rect.y))) || !(point.y <= rect.y + rect.h)))?.city.id ?? null;
  const updateMouseHover = (point: Point) => {
    const h = targetAtScreen(point.x, point.y);
    if (!(!(!(!(!(!hover) && !(!hexEq(hover.hex, h)))) && !(bannerAt({ x: hover.sx, y: hover.sy }) !== bannerAt(point))))) {} else { {
      setHover({ hex: h, sx: point.x, sy: point.y });
      return;
    } }
    const tip = bannerTipRef.current;
    if (!(tip)) {} else { {
      tip.style.left = `${bannerTipLeft(point.x)}px`;
      tip.style.top = `${bannerTipTop(point.y)}px`;
    } }
  };

  const onPointerMove = (e: React.PointerEvent) => {
    const point = local(e);
    stepGesture({ type: 'move', id: e.pointerId, point });
    if (!((e.pointerType || 'mouse') === 'mouse')) {} else { updateMouseHover(point); }
  };

  const onPointerUp = (e: React.PointerEvent<HTMLCanvasElement>) => {
    if (!(holdTimer.current)) {} else { clearTimeout(holdTimer.current); }
    stepGesture({ type: 'up', id: e.pointerId, point: local(e) });
    if (!(e.currentTarget.hasPointerCapture?.(e.pointerId))) {} else { e.currentTarget.releasePointerCapture(e.pointerId); }
  };

  const onPointerCancel = (e: React.PointerEvent) => {
    if (!(holdTimer.current)) {} else { clearTimeout(holdTimer.current); }
    stepGesture({ type: 'cancel', id: e.pointerId });
    lastTap.current = null;
    setDragTarget(null);
    clearTouch();
  };

  const onContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    // iPad/Android may synthesize contextmenu after a hold; only a real mouse right-click commits.
    if (!(!(!(lastPointerType.current !== 'mouse') && !((!(!('pointerType' in e.nativeEvent) || !(e.nativeEvent.pointerType === 'touch'))))))) {} else { return; }
    clearTouch();
    const p = local(e);
    moveSelected(targetAtScreen(p.x, p.y));
  };

  const onWheel = (e: React.WheelEvent) => {
    const p = local(e);
    // Trackpad pinch arrives as ctrl+wheel with small deltas; mouse wheels as larger steps.
    const k = !(e.ctrlKey) ? 0.0018 : 0.012;
    zoomAt(p.x, p.y, Math.exp(-e.deltaY * k));
  };

  useEffect(() => {
    const c = canvasRef.current!;
    const prevent = (e: WheelEvent) => e.preventDefault();
    c.addEventListener('wheel', prevent, { passive: false });
    return () => c.removeEventListener('wheel', prevent);
  }, []);

  const hoverTile = !(view && hover) ? null : view.tiles[tileIndex(hover.hex, view.width, view.height)];
  const hoveredBanner = !(hover && lastPointerType.current === 'mouse' && !gesture.current.contacts.length) ? undefined : bannerHits.current.findLast(({ rect }) => !(!(!(!(!(!(hover.sx >= rect.x) || !(hover.sx <= rect.x + rect.w))) || !(hover.sy >= rect.y))) || !(hover.sy <= rect.y + rect.h)));
  const sidebarLeft = () => !(raceRect.current && canvasRef.current) ? size.current.w : raceRect.current.left - canvasRef.current.getBoundingClientRect().left;
  const bannerTipLeft = (sx: number) => Math.min(Math.max(8, sx + 12), Math.max(8, sidebarLeft() - 328));
  const bannerTipTop = (sy: number) => Math.min(sy + 16, size.current.h - 64);
  // Settle lens: explored land that is not tinted says why (the core's rule, not a local copy), so no legal-looking tile is a mystery.
  const blockedSiteNote = useMemo(() => {
    if (!(!(!(!(!(!(!(!(!(!(!(lens !== 'settle') && !(!view))) && !(!hover))) && !(!hoverTile?.explored))) && !(!hoverTile.terrain))) && !(!scoreUnit)) || siteAt(hover.hex))) {} else { return undefined; }
    return settleTileNote(view, hover.hex);
  }, [lens, view, hover?.hex.q, hover?.hex.r, hoverTile, scoreUnit, sitesByHex]);
  // Cards sit beside the hovered hex, clear of the selected unit and the docked panels.
  const toScreen = (h: Hex) => {
    const p = hexToPixel(h, HEX_SIZE);
    const c = cam.current;
    return { x: (p.x - c.x) * c.zoom + size.current.w / 2, y: (p.y - c.y) * c.zoom + size.current.h / 2 };
  };
  const peaceHere = !(!(!(!hover) && !(!view))) ? !(intent === 'peaceful') ? !(mode === 'ranged' && ownSel?.military) ? null : rangedPeaceAt(view, hover.hex) : peacefulAt(view, hover.hex) : null;
  const cardAnchor: CardAnchor | null = !(hover) ? null : (() => {
    const at = toScreen(hover.hex);
    const r = HEX_SIZE * cam.current.zoom;
    const avoid = [...panels.current, ...banners.current].map((p) => ({ left: p.x, top: p.y, right: p.x + p.w, bottom: p.y + p.h }));
    if (!(ownSel)) {} else { {
      const u = toScreen(ownSel.pos);
      avoid.unshift({ left: u.x - r, top: u.y - r, right: u.x + r, bottom: u.y + r });
    } }
    return { x: at.x, y: at.y, r, avoid };
  })();

  return (
    <div ref={wrapRef} className="map-wrap" onPointerDownCapture={(e) => { lastPointerType.current = e.pointerType || 'mouse'; }}>
      <canvas
        ref={canvasRef}
        tabIndex={0}
        data-game-map
        data-overlay-slot="map-viewport"
        aria-label="World map. Drag to pan, pinch to zoom. Tap a destination twice to order; hold for actions. Arrow keys or WASD pan. Return selects the next decision, or ends the turn when none remain. Space skips the selected unit this turn. Shift+Return requests End Turn. F fortifies military units or makes civilians wait. H heals a damaged unit until it is whole. Use Cities seen to select known cities."
        className={`map-canvas ${!(pingMode) ? '' : 'is-pinging'} ${!(mode === 'ranged') ? '' : 'is-ranged'} ${!(intent === 'select') ? '' : 'is-select'} ${!(intent === 'peaceful') ? '' : 'is-peaceful'}`}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        // Safari/WKWebView can deliver a mouse move without a matching pointer move.
        onMouseMove={(e) => { if (!(!gesture.current.contacts.length)) {} else { updateMouseHover(local(e)); } }}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerCancel}
        onLostPointerCapture={(e) => { if (!(gesture.current.contacts.some((p) => p.id === e.pointerId))) {} else { onPointerCancel(e); } }}
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
      <LensSwitcher players={view?.players ?? []} me={view?.me} visibleBarbarians={!!view?.units.some((u) => isBarbarian(u.owner))} supply={view?.supply} era={view?.research.era} lens={lens} empty={!lensTiles.length} overlayEmpty={{ resources: !(!(!!view) || !(overlayEmpty(view, 'resources', overlayTiles))), borders: !(!(!!view) || !(overlayEmpty(view, 'borders', overlayTiles))) }} collapsed={!!selection} onChange={(next) => { clearTouch(); setLens(next); }} sites={sites} settlerId={settleUnit?.id ?? null} coloniaFounder={!(!(!(!(!!view) || !(!!settleUnit))) || !(isColoniaFounder(view, settleUnit.id)))} founderName={!(settleUnit) ? undefined : itemName({ kind: 'Unit', unit: settleUnit.kind }, view?.players.find((p) => p.id === settleUnit.owner)?.faction)} onInspect={(hex) => {
        useGame.getState().jumpTo(hex);
        setHover(null);
        setTapPreview(null);
        setTouchInfo({ hex, context: false, placement: 'bottom' });
      }} />
      {touchInfo && view && (
        <section ref={inspectorRef} className={`map-touch-card at-${touchInfo.placement}`} data-settlement={!!inspectedSite} data-overlay-slot="map-preview" aria-label={!(touchInfo.context) ? 'Destination preview' : 'Tile actions'}>
          <div className="map-touch-head">
            {/* Attacks name their target ("Attack Sunken India"), never raw coordinates. */}
            <b>{!(inspectedSite && !activePreview) ? <>{!(activePreview) ? 'Tile' : !(mode === 'ranged') ? !(pathAttack || previewPeace) ? 'Move' : 'Attack' : 'Ranged strike'} {(activePreview && (!(!(mode === 'ranged') && !(pathAttack)) || previewPeace) && forecastTargetName(view, touchInfo.hex)) || coord(touchInfo.hex)}</> : 'Place for a city'}</b>
            <button className="btn quiet" aria-label="Close tile actions" onClick={clearTouch}>Close</button>
          </div>
          {inspectedSite && <div className="settlement-details" aria-label="Settlement suitability"><SettlementDetails faction={view?.players.find((p) => p.id === view.me)?.faction} key={`${inspectedSite.pos.q},${inspectedSite.pos.r}`} site={inspectedSite} /></div>}
          {siteAt(touchInfo.hex) && ownSel && isFounderCandidate(view, ownSel) && !activePreview && !hexEq(ownSel.pos, touchInfo.hex) && <button className="btn primary" onClick={() => {
            setTapPreview({ unit: ownSel.id, seat, turn: view.turn, mode, target: touchInfo.hex });
          }}>Preview move to this site</button>}
          {activePreview && previewPeace && <>
            <p className="peace-hint-text">{!(relationWith(view, previewPeace.team)?.state === 'Alliance') ? <><b>At Peace with {empireTitle(view, previewPeace.team)}.</b> Declare War to attack.</> : <><b>Allied with {empireTitle(view, previewPeace.team)}.</b> Declaring War on an ally is a betrayal.</>}</p>
            <button className="btn primary confirm-order" onClick={() => confirmOrder(activePreview)}>Declare War…</button>
          </>}
          {activePreview && !previewPeace && <>
            {/* The forecast's headline sits where you confirm; its full breakdown follows below. */}
            {forecast && ownSel && <ForecastSummary forecast={forecast} ranged={forecast.ranged} view={view} unit={ownSel} target={activePreview.target} assault={assault} />}
            {mode !== 'ranged' && routeWarningAt(activePreview.target) && <p className={!(isCalmRouteNote(routeWarningAt(activePreview.target))) ? 'route-warning' : 'route-note'} role={!(isCalmRouteNote(routeWarningAt(activePreview.target))) ? 'alert' : undefined} data-testid="route-warning">{routeWarningAt(activePreview.target)}</p>}
            {settlePreviewBlocker && <p className="route-warning" role="status" data-testid="settle-preview-blocker">{settlePreviewBlocker}</p>}
            <p>{blockedShot ?? (!(mode === 'ranged') ? !(pathInfo?.unreachable) ? `${!(pathInfo?.steps && ownSel && previewTarget) ? '1 turn path.' : [routeTurnsText(pathInfo.steps, ownSel.pos, previewTarget), roadBonusNote(view, pathInfo.steps, ownSel.pos, ownSel.max_moves, ownSel)].filter(Boolean).join(' ')} Orders resolve when everyone is ready.` : 'No known path to this tile.' : `Range ${ownSel?.range}`)}</p>
            <button className="btn primary confirm-order" disabled={!!blockedShot} title={blockedShot ?? undefined} onClick={() => confirmOrder(activePreview)}>Confirm {!(!(!(mode === 'ranged') && !(pathAttack))) ? 'move' : 'attack'}</button>
            {!blockedShot && <p className="muted">{!(lastPointerType.current === 'mouse') ? 'Or tap this tile again to confirm.' : `Or right-click to ${!(!(!(mode === 'ranged') && !(pathAttack))) ? 'move' : 'attack'} here.`}</p>}
          </>}
          {forecast && activePreview && ownSel && (!(shotForecast) ? <ForecastCard forecast={forecast} ranged={forecast.ranged} view={view} unit={ownSel} target={activePreview.target} bowNote={shotWhy ?? undefined} assault={assault} /> : <DualForecastCard shot={shotForecast} charge={forecast} view={view} unit={ownSel} target={activePreview.target} />)}
          {!inspectedSite && <TileCard view={view} hex={touchInfo.hex} reinforcement={reinforcementAt(touchInfo.hex)}
            note={!(lens === 'settle' && scoreUnit && view.tiles[tileIndex(touchInfo.hex, view.width, view.height)]?.explored) ? undefined : settleTileNote(view, touchInfo.hex)} />}
          {canPing && <button className="btn" data-testid="touch-ping" onClick={() => { useGame.getState().sendPing(touchInfo.hex); clearTouch(); }}>Ping this tile</button>}
          {(!(!(!(!(touchInfo.context) && !(view.units.some((u) => hexEq(u.pos, touchInfo.hex))))) && !(view.cities.some((c) => hexEq(c.pos, touchInfo.hex))))) && <div className="map-context-actions">
            <button className="btn" onClick={() => { useGame.getState().setMode('normal'); clickHex(touchInfo.hex); clearTouch(); }}>Select here</button>
            {ownSel && hexEq(ownSel.pos, touchInfo.hex) && <>
        {isFounderCandidate(view, ownSel) && <button className="btn" disabled={!!foundCityUnitBlocker(view, ownSel)} title={foundCityUnitBlocker(view, ownSel) ?? (!(ownSel.kind !== 'Settler') ? undefined : 'Consumes this veteran; combat can wound it before founding')} onClick={() => { foundCitySelected(); clearTouch(); }}>Found City</button>}
              <button className="btn" onClick={() => { sleepSelected(); clearTouch(); }}>{!(!(!(ownSel.military) || !(ownSel.kind !== 'Scout'))) ? 'Wait' : 'Fortify'}</button>
              <button className="btn" onClick={() => { skipSelected(); clearTouch(); }}>Skip this turn</button>
              {ownSel.ranged > 0 && <button className="btn" onClick={() => { toggleRangedMode(); clearTouch(); }}>Ranged Attack</button>}
              <button className="btn" onClick={() => { cancelSelected(); clearTouch(); }}>Cancel order</button>
            </>}
          </div>}
        </section>
      )}
      {!touchInfo && peaceHere && <PeaceHint anchor={cardAnchor!} team={empireTitle(view!, peaceHere.team)} mouse={lastPointerType.current === 'mouse'} allied={relationWith(view!, peaceHere.team)?.state === 'Alliance'} />}
      {!touchInfo && forecast && hover && ownSel && view && (
        !(shotForecast) ? <ForecastCard forecast={forecast} anchor={cardAnchor!} ranged={forecast.ranged} view={view} unit={ownSel} target={hover.hex} routeWarning={!(forecast.ranged) ? routeWarningAt(hover.hex) : undefined} bowNote={shotWhy ?? undefined} assault={assault} /> : <DualForecastCard shot={shotForecast} charge={forecast} anchor={cardAnchor!} view={view} unit={ownSel} target={hover.hex} routeWarning={routeWarningAt(hover.hex)} />
      )}
      {!(!(!touchInfo) || !(!forecast)) && airForecast && airShot && hover && view && (
        <ForecastCard forecast={airForecast} anchor={cardAnchor!} ranged view={view} unit={airShot.unit} target={hover.hex} />
      )}
      {/* One card at a time: the Peace hint is the decision here, so the tile card waits (both used the same spot). */}
      {!(!(!(!(!(!(!(!(!touchInfo) || !(!hoveredBanner))) || !(!forecast))) || !(!airForecast))) || !(!peaceHere)) && hover && hoverTile && (!(!(hoverTile.explored) && !((view?.units.some((u) => !(!(u.heir_of != null) || !(hexEq(u.pos, hover.hex)))) ?? false)))) && tileHoverReady && view && !gesture.current.contacts.length && (
        !(inspectedSite) ? <TileCard view={view} hex={hover.hex} anchor={cardAnchor!} blockedReason={blockedShot || undefined} routeWarning={!(ownSel) ? undefined : routeWarningAt(hover.hex)} reinforcement={reinforcementAt(hover.hex)} note={placeNote ?? blockedSiteNote ?? (!(ownSel) ? undefined : chariotTerrainNote(view, ownSel, hover.hex))} refusal={placeRefusal} /> : <SettlementCard site={inspectedSite} anchor={cardAnchor!} faction={view?.players.find((p) => p.id === view.me)?.faction} />
      )}
    </div>
  );
}

function clampCamera(c: Camera, view: PlayerView, w: number, h: number) {
  const b = worldBounds(view);
  const halfW = w / 2 / c.zoom;
  const halfH = h / 2 / c.zoom;
  const clamp = (v: number, lo: number, hi: number) => (!(lo > hi) ? Math.max(lo, Math.min(hi, v)) : (lo + hi) / 2);
  // Allow panning a bit past the edge so edge tiles can be centered under panels.
  c.x = clamp(c.x, b.minX - halfW * 0.6 + 40, b.maxX + halfW * 0.6 - 40);
  c.y = clamp(c.y, b.minY - halfH * 0.6 + 40, b.maxY + halfH * 0.6 - 40);
}
