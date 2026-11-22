import { celebrationFrame, type MapCelebration } from '../celebrate/mapEffect';
// Map renderer. The painterly terrain (terrain.ts) is cached in chunks per zoom level; this file
// draws it and then the per-frame layers: movement reach, hover, lettering, routes, selection,
// unit tokens and city banners. Tokens, banners and text are drawn in screen space so they stay
// crisp at any zoom.
import { noteMapDraw } from '../frameTelemetry';
import type { CityBuildFocus, CityView, Faction, Hex, PlayerView, ProductionItem, UnitKind, UnitView } from '../types';
import { growthProgress, productionProgress } from '../cityProgress';
import type { GoalPreview } from '../goalActions';
import { itemIcon, sameItem } from '../data';
import { isFighter } from '../airText';
import { capitalStar } from '../capitalMark';
import { hexToPixel, tileIndex } from '../hex';
import { healthColor, rgba } from '../theme';
import type { PathStep } from '../path';
import { tintedIcon } from '../icons';
import type { LookId } from '../brand/look';
import { MAP_STYLES, mapBorderColor, mapPlayerColor, type MapStyle } from './style';
import { TerrainCache } from './terrain';
import { getBorderVariant } from './borderVariant';
import { getTerrainVariant } from './terrainVariant';
import { BANNER_BOTTOM, CITY_UNIT_DROP, HEX_SIZE, K, PIP_HALF, sealRadius, boundary, centerX, centerY, hexPath, isFar, neighborIndex, tokenHalf, tracePolylines } from './grid';
import { computeLabels, type MapLabel } from './labels';
import { reachTiles } from './reach';
import type { MapCue, ReportArrow } from './cues';
import { drawSiegeBadge, drawSiegeBar, drawSiegeRing, drawSiegeSealRing, SIEGE_BAR_GAP, SIEGE_BAR_H, siegeRingBox, siegeSealRingBox } from './siegeMark';
import type { SiegeLevel } from '../siege';
import { miniatureImage } from '../unitArt';
import { siegePropFor, siegePropImage, siegePropRect, type SiegeProp } from '../siegeProps';
import { attackableEnemies, drawEraPlaque, drawStrengthChip, matchupStrength, plaqueHeight, unitEra } from './unitEra';
import { contractLeft } from '../company';
import { heirTag } from '../heir';
import { rankLevel } from '../rankInsignia';
import { FONTS, type BannerSpec, bannerMetersSize, bannerRect, drawBadge, drawBanner, drawBannerMeters, type BannerMeter, barbarianMiniature, drawBarbarianToken, drawContractPip, drawContractTag, drawHeirHalo, drawLabel, drawPip, drawPromotion, drawRankInsignia, drawSprite, drawTag, drawToken, drawTokenHealth, miniatureGeometry, nameSprite, sealSprite, TOKEN_INK, capitalSealRadius } from './sprites';
import { mapFeatureImage } from './mapFeatures';
import { BARBARIAN, isBarbarian } from '../barbarians';
import type { Rect } from './compass';
import type { LensTile, MapLens } from './lenses';
import { drawLens, drawOwnerOverlay } from './drawLens';
import { artFadeFor, DEFAULT_OVERLAYS, ownerWashScale, type OverlaySet } from './overlays';
import { drawYieldIcons, yieldClusterRects } from './yieldIcons';
import { drawImprovements } from './improvementArt';
import { drawResourceIcons, medallionTop, resourceDetail, STACK_LIFT } from './resourceIcons';
import type { ScenarioTarget } from '../scenarios/scenarios';
import { drawRoads } from './roadArt';
import { drawClosedLand } from './sakokuLand';
import { drawHallBadge, drawHallLand, hallBadgeAt, hallSites } from './courierHall';
import { drawWonderArt, drawWonderBadges } from './wonderArt';
import { deathPos, hpAt, type CombatPlan, type FxGhost } from './combatFx';
import { aircraftPassProgress, drawCombatFx, drawSiegeAmbient, siegeAmbientBox } from './combatFxDraw';
import { smoulderVisible } from './smoulder';
import type { March, MomentHold, MomentPlan } from './momentFx';
import { drawChimneySmoke, drawWaterGlints, idleBoxes, type IdleView } from './idleLife';
import { drawLandmarks, LANDMARK_REF_ZOOM, landmarkBoxes, landmarksOf, townCover } from './landmarks';
import { drawConstructionSites } from './constructionFx';
import { constructionStage, sitesOf, type ConstructionSite } from './constructionSites';
import { CONSTRUCTION_ANIMATION, buildPhases } from './momentFx';
import { drawMarchDust, drawMomentFx, drawMomentGround, drawRevealed, revealAt, type MomentView } from './momentFxDraw';
import { drawAirCandidates, drawAirLabels, drawAirOffscreen, drawAirRings, drawTileCandidates, type AirOverlay } from './airOverlay';
import { aircraftAsUnit } from '../airWing';
import { unworkedClaimIndexes } from './unworkedClaims';

export { HEX_SIZE } from './grid';

export interface Camera {
  x: number;
  y: number;
  zoom: number;
}

/** How far to raise a token that stands behind a zoomed-in city banner (on a hex above the city): its centre
 * goes up to the banner's top edge, so the head and owner disc show above the banner, by at most half a hex
 * (the token stays on its own hex). Tokens below a banner's bottom edge, or clear of it, never move. */
export function bannerLift(x: number, y: number, tw: number, hexPx: number, covers: readonly Rect[]): number {
  let lift = 0;
  for (const c of covers) {
    if (x + tw <= c.x || x - tw >= c.x + c.w || y >= c.y + c.h || y + tw * 1.3 <= c.y) continue;
    lift = Math.max(lift, y - c.y);
  }
  return Math.min(lift, hexPx * 0.5);
}

export function neutralBarbarianCue(view: Pick<PlayerView, 'me' | 'players'>, owner: number): boolean {
  return isBarbarian(owner) && view.players.find((p) => p.id === view.me)?.faction === 'Celts';
}

/** `invalid`: the queued order will fail (shown on the unit instead of its order line). */
export type PendingMark = 'explore' | 'move' | 'sleep' | 'heal' | 'skip' | 'found' | 'disband' | 'ranged' | 'upgrade' | 'invalid';

export interface RenderInput {
  view: PlayerView;
  cityFocuses?: ReadonlyMap<number, import('../types').CityBuildFocus | null>;
  /** What each own city was told to build this turn (an order not yet resolved), so its banner shows the choice at once. */
  cityBuilds?: ReadonlyMap<number, ProductionItem>;
  cam: Camera;
  width: number;
  height: number;
  dpr: number;
  time: number;
  /** The clock the selection pulse reads, when it must run ahead of `time` (a pulse-only frame freezes `time` so the rest of the map does not move). */
  pulseTime?: number;
  hover: Hex | null;
  lens?: MapLens;
  lensTiles?: LensTile[];
  /** The stackable overlays (Yields, Resources, Borders) on top of whichever lens is showing. */
  overlays?: OverlaySet;
  /** Their tiles: yield clusters, and owned land for the Borders overlay. */
  yieldTiles?: LensTile[];
  borderTiles?: LensTile[];
  /** Screen rects of panels over the map: lens badges move clear of them. */
  lensAvoid?: readonly Rect[];
  /** Battle result cues (damage, losses) over hexes, already faded for this frame. */
  cues?: MapCue[];
  /** A clicked combat line: attacker → target arrow (or a target ring when the attacker went unseen). */
  reportArrow?: ReportArrow | null;
  celebration?: MapCelebration;
  /** The hovered tile holds the player's own unit or city: draw a select ring, not a path. */
  hoverSelect?: boolean;
  selectedUnit: number | null;
  selectedAir?: number | null;
  selectedCity: number | null;
  path: PathStep[] | null;
  pathAttack: boolean;
  pathUnreachable: Hex | null;
  /** A friend on this turn's route that may stop it (path.ts:previewRoute). */
  pathRisk?: Hex | null;
  pendingLines: { from: Hex; to: Hex; attack: boolean }[];
  pendingMarks: Map<number, PendingMark>;
  rangedTiles: { valid: Hex[]; blocked: Hex[] } | null;
  /** Growth claim highlighted while the city territory bar is hovered. */
  territory?: TerritoryOverlay | null;
  /** The selected own city's improvable tiles (map-first picker for the Improvements group). */
  improve?: ImproveOverlay | null;
  /** A tile-chosen build being placed: every eligible tile, and the one under the pointer. */
  place?: { candidates: Hex[]; hover: Hex | null; /** Buying a tile: a price on every buyable tile, greyed when unaffordable; the rest of the map is dimmed. */ prices?: { tile: Hex; price: number; affordable: boolean }[] } | null;
  /** Tiles with a queued purchase, each marked with its price. */
  tileBuys?: { tile: Hex; price: number }[];
  /** The selected own city's aircraft: their range rings, and the open strike or rebase picker's candidates. */
  air?: AirOverlay | null;
  /** Great Migration, weighed or ordered: the cities that shrink (ringed) and the tiles they release (washed). */
  goalPreview?: GoalPreview | null;
  /** Animated position overrides in world pixels. */
  unitPos: Map<number, { x: number; y: number }>;
  /** Combat playback (combatFx.ts): world-pixel offsets for lunges and flinches, units that fell this turn, and the effects at `t` ms. */
  unitShift?: Map<number, { x: number; y: number }>;
  ghosts?: FxGhost[];
  /** Marching units (momentFx.ts `marchOf`): a stride lift (world px) and the dust off their heels. */
  march?: Map<number, March>;
  fx?: { plan: CombatPlan; t: number } | null;
  /** Idle life (idleLife.ts): smoke over the bigger towns and glints on the water, at `time`. */
  idle?: boolean;
  /** Payoff moments (momentFx.ts) at `t` ms, and what the map holds back for them. */
  moments?: { plan: MomentPlan; t: number; hold: MomentHold } | null;
  /** Map animations on: damaged cities smoulder and burn (combatFxDraw.ts drawSiegeAmbient), on this clock (ms). */
  ambientT?: number | null;
  /** Idle life's clock (idleLife.ts), when it differs from `time`: an idle tick advances only idle life. */
  idleT?: number;
  seatColor: string;
  /** The scenario objective's cities, ringed even in fog. */
  scenarioTargets?: ScenarioTarget[];
  /** Own cities under attack (city id → how soon it falls): drawn alarmed (siegeMark.ts). */
  siege?: ReadonlyMap<number, SiegeLevel>;
  reducedMotion: boolean;
}

/** What the map marks for the city panel's Improvements group. */
export interface ImproveOverlay {
  /** Every tile the city could improve now. */
  candidates: Hex[];
  /** Tiles the city is building or has queued an improvement on. */
  building: Hex[];
  /** Tiles a placed Yam Relay Station or Great Work will stand on: a gilt double ring, unlike any border. */
  sites: Hex[];
  /** The tile whose picker row is hovered or focused, and the one a map click chose. */
  hover: Hex | null;
  pick: Hex | null;
}

/** Growth claim highlighted while the city territory bar is hovered. */
export interface TerritoryOverlay {
  /** The tile the city's next growth claim takes, highlighted while the territory bar is hovered. */
  claim: Hex | null;
}

// ---- caches: one terrain cache per (look, seat, map, dpr) ----

const caches = new Map<string, TerrainCache>();
let lastView: PlayerView | null = null;
let labelCache: { view: PlayerView; labels: MapLabel[] } | null = null;

function terrainFor(view: PlayerView, style: MapStyle, dpr: number): TerrainCache {
  const key = `${style.id}|${view.me}|${view.width}x${view.height}|${dpr}|${getBorderVariant()}|${getTerrainVariant()}`;
  let c = caches.get(key);
  if (!c) {
    // Keep at most the two seats of the current look and map.
    for (const k of [...caches.keys()]) if (!k.startsWith(`${style.id}|`) || !k.includes(`|${view.width}x${view.height}|${dpr}|`)) caches.delete(k);
    const t0 = performance.now();
    c = new TerrainCache(view, style, dpr);
    perf.buildMs = performance.now() - t0;
    caches.set(key, c);
  } else if (view !== lastView) {
    const t0 = performance.now();
    c.update(view);
    perf.updateMs = performance.now() - t0;
  }
  lastView = view;
  return c;
}

/** The newest terrain cache's chunk bookkeeping after its last draw (tests and the perf overlay read it). */
export function terrainStats(): TerrainCache['lastStats'] | null {
  let last: TerrainCache | null = null;
  for (const c of caches.values()) last = c;
  return last ? { ...last.lastStats } : null;
}

function labelsFor(view: PlayerView): MapLabel[] {
  if (!labelCache || labelCache.view !== view) labelCache = { view, labels: computeLabels(view) };
  return labelCache.labels;
}

export function worldToScreen(cam: Camera, w: number, h: number, x: number, y: number) {
  return { x: (x - cam.x) * cam.zoom + w / 2, y: (y - cam.y) * cam.zoom + h / 2 };
}

export function screenToWorld(cam: Camera, w: number, h: number, sx: number, sy: number) {
  return { x: (sx - w / 2) / cam.zoom + cam.x, y: (sy - h / 2) / cam.zoom + cam.y };
}

/** Map size in world pixels, for camera clamping. */
export function worldBounds(view: PlayerView) {
  const hw = HEX_SIZE * Math.sqrt(3);
  return {
    minX: -HEX_SIZE,
    minY: -HEX_SIZE,
    maxX: hw * (view.width - 0.5) + HEX_SIZE,
    maxY: HEX_SIZE * 1.5 * (view.height - 1) + HEX_SIZE,
  };
}

const ROMAN = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X', 'XI', 'XII'];
const roman = (n: number) => ROMAN[n - 1] ?? String(n);

// ---- performance probe (read by tests and the perf script via window.legatumMap.perf) ----
export const perf = { frames: 0, partialFrames: 0, drawMs: 0, maxMs: 0, workMs: 0, buildMs: 0, updateMs: 0, measureFrames: false, fxFrames: 0, fxMs: 0, fxMaxMs: 0, momentFrames: 0, momentMs: 0, momentMaxMs: 0, reset() { this.frames = 0; this.partialFrames = 0; this.drawMs = 0; this.maxMs = 0; this.workMs = 0; this.fxFrames = 0; this.fxMs = 0; this.fxMaxMs = 0; this.momentFrames = 0; this.momentMs = 0; this.momentMaxMs = 0; } };

// ---- movement reach for the selected unit ----

let reachCache: { key: string; tiles: Set<number> } | null = null;
function reachOf(view: PlayerView, u: UnitView): Set<number> {
  const key = `${u.id}|${u.pos.q},${u.pos.r}|${u.max_moves}|${view.turn}|${view.me}`;
  if (reachCache && reachCache.key === key && lastView === view) return reachCache.tiles;
  const tiles = reachTiles(view, u);
  reachCache = { key, tiles };
  return tiles;
}

const NO_SIEGE: ReadonlyMap<number, SiegeLevel> = new Map();

// ---- main draw ----

/** Draw a frame, returning visible label bounds and whether terrain needs another frame. */
export function draw(ctx: CanvasRenderingContext2D, inp: RenderInput): { pending: boolean; labelRects: Rect[]; bannerRects: Rect[]; bannerHits: { city: CityView; rect: Rect; cover: Rect; parts: Rect[]; far: boolean }[]; liftedHits: { hex: Hex; rect: Rect }[]; idle: boolean; ring: SelectionRing | null; ambient: Rect[] } {
  const t0 = performance.now();
  const { view, cam, width: w, height: h, dpr } = inp;
  const look: LookId = 'atlas';
  const S = MAP_STYLES[look];
  const z = cam.zoom;
  const hexPx = HEX_SIZE * z;
  const factionOf = new Map<number, Faction>(view.players.map((pl) => [pl.id, pl.faction]));

  const terrain = terrainFor(view, S, dpr);
  const overlays = inp.overlays ?? DEFAULT_OVERLAYS;
  terrain.setArtFade(artFadeFor(inp.lens, overlays));
  const moments = inp.moments ?? null;
  terrain.setHold(moments?.hold ?? null);
  const pending = terrain.draw(ctx, cam, w, h, 7);
  // During combat playback the HP bars drop as each blow lands (combatFx.ts), not all at once.
  const fxNow = inp.fx;
  const cityHp = (c: CityView) => (fxNow ? hpAt(fxNow.plan, `c${c.id}`, fxNow.t) : undefined) ?? c.hp;
  const idleView: IdleView | null = inp.idle ? { toScreen: (p) => worldToScreen(cam, w, h, p.x, p.y), hexPx, far: isFar(hexPx), w, h, dpr } : null;
  const idleT = inp.idleT ?? inp.time;
  let idleLive = idleView ? drawWaterGlints(ctx, view, idleT, idleView) : 0;
  const siege = inp.siege ?? NO_SIEGE;
  // A captured city's banner keeps its old owner until it flips (momentFx.ts).
  const held = moments?.hold.cityOwner;
  const bannerView = held?.size ? { ...view, cities: view.cities.map((c) => held.has(c.id) ? { ...c, owner: held.get(c.id)! } : c) } : view;
  const focusOf = (c: CityView) => c.owner === view.me ? (inp.cityFocuses?.has(c.id) ? inp.cityFocuses.get(c.id) : c.detail?.build_automation) ?? null : null;
  const banners = cityBanners(bannerView, cam, w, h, dpr, look, inp.selectedCity, siege, focusOf, moments?.hold.unscored, inp.cityBuilds);
  // What labels, badges and medallions keep clear of: the banners as drawn this frame (screen px).
  const bannerCovers = banners.map((b) => b.cover);
  const toScreen = (p: { x: number; y: number }) => worldToScreen(cam, w, h, p.x, p.y);
  const landmarkSites = [...landmarksOf(view), ...sitesOf(view).filter((s) => !s.walls), ...(moments?.plan.builds.filter((b) => b.wonder).map((b) => ({ tile: b.tile, hex: b.at })) ?? [])];
  // Landmarks use a fixed footprint chosen from every banner at one reference zoom (never these screen covers, never moving units).
  const fixedLandmarkBoxes = landmarkBoxes(landmarkSites, toScreen, hexPx, referenceBannerCovers(bannerView, look, siege));
  // The ink-and-wash landmark sits in the land, under everything else the map draws (full colour belongs to its cards).
  drawWonderArt(ctx, view, cam, w, h, dpr, inp.lens, overlays.yields);
  // Improvements change the land itself: drawn on it, under every lens, overlay, unit and banner.
  const momentView: MomentView | null = moments ? {
    toScreen: (p) => worldToScreen(cam, w, h, p.x, p.y), hexPx, zoom: z, dpr, far: isFar(hexPx), w, h, colorOf: mapBorderColor,
    eraOf: (owner) => Math.max(0, (view.players.find((p) => p.id === owner)?.era ?? 1) - 1),
    factionOf: (owner) => factionOf.get(owner),
    city: (id) => { const c = view.cities.find((x) => x.id === id); return c ? { population: c.population, owner: c.owner } : null; },
    unitAt: (id) => { const u = view.units.find((x) => x.id === id); return u ? inp.unitPos.get(id) ?? hexToPixel(u.pos, HEX_SIZE) : null; },
    tokenHalf: isFar(hexPx) ? PIP_HALF : tokenHalf(hexPx),
    landmarkAt: (city, building) => { const l = landmarksOf(view).find((x) => x.city === city && x.building === building); return l ? hexToPixel(l.hex, HEX_SIZE) : null; },
    landmarkBoxes: fixedLandmarkBoxes,
  } : null;
  // New improvements and roads draw themselves in (momentFx.ts): the land from before outside what they have reached.
  const reveal = moments && momentView ? revealAt(moments.plan, moments.t, view, momentView) : null;
  if (reveal?.improves) drawRevealed(ctx, dpr, w, h, reveal.improves, view, (v) => drawImprovements(ctx, v, cam, w, h, dpr, look));
  else drawImprovements(ctx, view, cam, w, h, dpr, look);
  // Constructed wonders stand on their chosen tile (landmarks.ts); one being built this turn waits for its construction.
  const building = moments && CONSTRUCTION_ANIMATION ? moments.plan.builds.filter((b) => b.wonder && moments.t < b.t + buildPhases(b).done) : [];
  drawLandmarks(ctx, view, toScreen, hexPx, w, h, dpr, building.length ? (l) => building.some((b) => b.city === l.city) : undefined, fixedLandmarkBoxes);
  // A completion removes the site from the after-view. Keep its old state until the finish begins,
  // then let the moment draw it alone; otherwise two semi-transparent cranes ghost over each other.
  const activeSites = !CONSTRUCTION_ANIMATION ? [] : moments?.plan.builds.filter((b) => (b.wonder || b.walls) && moments.t >= b.t && moments.t < b.t + buildPhases(b).done) ?? [];
  const currentSiteCities = new Set(sitesOf(view).map((s) => s.city));
  const oldSites: ConstructionSite[] = !CONSTRUCTION_ANIMATION ? [] : moments?.plan.builds.filter((b) => (b.wonder || b.walls) && b.to >= 1 && moments.t < b.t && !currentSiteCities.has(b.city)).map((b) => ({
    city: b.city, owner: b.owner, tile: b.tile, hex: b.at,
    building: b.item.kind === 'Building' || b.item.kind === 'Wonder' ? b.item.building : 'Walls',
    era: Math.max(0, (view.players.find((p) => p.id === b.owner)?.era ?? 1) - 1),
    k: b.from, stage: constructionStage(b.from), seed: b.seed, walls: b.walls,
  })) ?? [];
  drawConstructionSites(ctx, view, toScreen, hexPx, w, h, dpr, moments?.hold.sites, fixedLandmarkBoxes, oldSites, new Set(activeSites.map((b) => b.city)));
  // A town being founded or growing, walls going up, land changing hands: drawn into the land, under everything else.
  if (moments && momentView) drawMomentGround(ctx, moments.plan, moments.t, momentView);
  if (idleView) idleLive += drawChimneySmoke(ctx, view, idleT, idleView, moments ? (tile) => moments.hold.towns.has(tile) : undefined);
  if (inp.lensTiles?.length) drawLens(ctx, inp.lensTiles, cam, w, h, view.width, dpr, look);
  // Overlays draw over the lens tint, so a lens never hides them: owned land first, then the yield icons on top of both.
  if (inp.borderTiles?.length) drawOwnerOverlay(ctx, inp.borderTiles, cam, w, h, view.width, dpr, look, 'tint', ownerWashScale(inp.lens));
  // Tile yields sit under every unit and banner: quiet enough to leave on.
  if (inp.yieldTiles?.length) drawYieldIcons(ctx, inp.yieldTiles, cam, w, h, view.width, dpr, look, inp.lens === 'settle');
  // Banner positions are known up front so the resource medallions below them can keep clear.
  const bannerMotion = moments?.hold.banner;
  // Known resources, on every lens (as in Civ): top of the hex, under units and banners.
  const iconView = moments?.hold.resources.size
    ? { ...view, tiles: view.tiles.map((tile, i) => moments.hold.resources.has(i) ? { ...tile, resource: null } : tile) }
    : view;
  drawResourceIcons(ctx, iconView, cam, w, h, dpr, look, bannerCovers, overlays.resources);
  // A natural wonder's bonus, on its tile: above the lens tint and yield icons, under units and banners.
  drawWonderBadges(ctx, view, cam, w, h, dpr, overlays.yields, bannerCovers);

  // World transform for vector overlays.
  const worldT = () => ctx.setTransform(dpr * z, 0, 0, dpr * z, dpr * (w / 2 - cam.x * z), dpr * (h / 2 - cam.y * z));
  const screenT = () => ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  const W = view.width;
  const H = view.height;
  const hxCenter = (hx: Hex) => hexToPixel(hx, HEX_SIZE);
  const S2 = (hx: Hex) => {
    const p = hxCenter(hx);
    return worldToScreen(cam, w, h, p.x, p.y);
  };
  const px = 1 / z;

  worldT();
  const selUnit = inp.selectedUnit !== null ? view.units.find((u) => u.id === inp.selectedUnit) ?? null : null;
  const ownSel = selUnit && selUnit.owner === view.me ? selUnit : null;

  // Movement reach: a wash plus an edge.
  if ((!inp.lens || inp.lens === 'default' || inp.lens === 'movement') && ownSel && !inp.rangedTiles && ownSel.max_moves > 0 && !inp.unitPos.size) {
    const reach = reachOf(view, ownSel);
    if (reach.size) {
      ctx.beginPath();
      for (const i of reach) hexPath(ctx, centerX(i, W), centerY(i, W), HEX_SIZE - 2 * K);
      ctx.fillStyle = S.ink.reachFill;
      ctx.fill();
      const start = tileIndex(ownSel.pos, W, H);
      const lines = boundary([...reach, start], (i) => reach.has(i) || i === start, W, H);
      ctx.beginPath();
      tracePolylines(ctx, lines);
      ctx.strokeStyle = S.ink.reachEdge;
      ctx.lineWidth = 1.2 * px;
      ctx.lineCap = 'round';
      ctx.setLineDash(S.ink.reachDash ? S.ink.reachDash.map((d) => d * px) : []);
      ctx.stroke();
      ctx.setLineDash([]);
    }
  }

  // The aircraft in the selected city: each range as a ring (the labels follow in screen space).
  if (inp.air?.rings.length) {
    drawAirRings(ctx, inp.air, W, H, inp.seatColor, px);
    // Labels under every token and banner: screen space, then back to the world transform.
    screenT();
    drawAirLabels(ctx, inp.air, (x, y) => worldToScreen(cam, w, h, x, y), inp.seatColor, FONTS[look].sans, { w, h }, [...(inp.lensAvoid ?? []), ...bannerCovers]);
    worldT();
  }

  // Population is territory: tiles an own city has claimed beyond its work radius are owned but
  // not worked (they only block rivals). They read as yours: player-colour hatch lines and a
  // dashed outline around the whole unworked claim; zoomed out, a flat wash and a lighter outline.
  {
    const unworked = unworkedClaimIndexes(view, W, H);
    if (unworked.size) {
      const own = inp.seatColor;
      const flat = hexPx < 24;
      ctx.save();
      if (flat) {
        ctx.fillStyle = rgba(own, 0.25);
        ctx.beginPath();
        for (const i of unworked) hexPath(ctx, centerX(i, W), centerY(i, W), HEX_SIZE);
        ctx.fill();
      } else {
        ctx.strokeStyle = rgba(own, 0.35);
        ctx.lineWidth = px * 1.2;
        for (const i of unworked) {
          const cx = centerX(i, W), cy = centerY(i, W);
          ctx.save();
          ctx.beginPath();
          hexPath(ctx, cx, cy, HEX_SIZE * 0.96);
          ctx.clip();
          ctx.beginPath();
          const step = 7 * px, r = HEX_SIZE;
          for (let o = -2 * r; o <= 2 * r; o += step) {
            ctx.moveTo(cx + o - r, cy + r);
            ctx.lineTo(cx + o + r, cy - r);
          }
          ctx.stroke();
          ctx.restore();
        }
      }
      ctx.beginPath();
      tracePolylines(ctx, boundary(unworked, (i) => unworked.has(i), W, H, HEX_SIZE * 0.96));
      ctx.strokeStyle = rgba(own, flat ? 0.5 : 0.85);
      ctx.lineWidth = (flat ? 1 : 1.6) * px;
      ctx.setLineDash(flat ? [3 * px, 4 * px] : [5 * px, 3 * px]);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.restore();
    }
  }

  // Selected city: its territory outline. Every owned tile inside the work radius is worked (the hatch
  // marks the claimed-only ones beyond it), so a per-tile "worked" mark said nothing the hatch didn't.
  const selCity = inp.selectedCity !== null ? view.cities.find((c) => c.id === inp.selectedCity) : null;
  if (selCity?.detail) {
    const set = new Set(selCity.detail.territory.map((hx) => tileIndex(hx, W, H)).filter((i) => i >= 0));
    ctx.beginPath();
    tracePolylines(ctx, boundary(set, (i) => set.has(i), W, H, HEX_SIZE * 0.92));
    ctx.strokeStyle = TOKEN_INK.atlas.ink;
    ctx.lineWidth = 1.4 * px;
    ctx.setLineDash([5 * px, 3 * px]);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  const T = inp.territory;
  // Highlight the tile a city will claim when its population grows.
  if (T?.claim) {
    const p = hxCenter(T.claim);
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.84);
    ctx.strokeStyle = S.tone.growth;
    ctx.lineWidth = 3 * px;
    ctx.setLineDash([]);
    ctx.stroke();
  }

  // Improvements: the selected city's improvable tiles are outlined (dashed, translucent); tiles it is
  // already improving read solid; the hovered row's tile and the clicked tile are stronger.
  const I = inp.improve;
  if (I) {
    const own = inp.seatColor;
    const same = (a: Hex | null | undefined, b: Hex) => !!a && a.q === b.q && a.r === b.r;
    ctx.save();
    for (const hx of I.candidates) {
      if (same(I.hover, hx) || same(I.pick, hx)) continue; // drawn above the counters, below
      const p = hxCenter(hx);
      ctx.beginPath();
      hexPath(ctx, p.x, p.y, HEX_SIZE * 0.88);
      ctx.strokeStyle = rgba(own, 0.6);
      ctx.lineWidth = 1.4 * px;
      ctx.setLineDash([2 * px, 3 * px]);
      ctx.stroke();
    }
    for (const hx of I.building) {
      const p = hxCenter(hx);
      ctx.beginPath();
      hexPath(ctx, p.x, p.y, HEX_SIZE * 0.88);
      ctx.strokeStyle = rgba(own, 0.9);
      ctx.lineWidth = 2 * px;
      ctx.setLineDash([]);
      ctx.stroke();
    }
    ctx.setLineDash([]);
    for (const hx of I.sites) {
      const p = hxCenter(hx);
      ctx.beginPath();
      hexPath(ctx, p.x, p.y, HEX_SIZE * 0.84);
      ctx.fillStyle = rgba(S.tone.gold, 0.16);
      ctx.fill();
      ctx.strokeStyle = rgba(S.tone.paper, 0.95);
      ctx.lineWidth = 5 * px;
      ctx.stroke();
      ctx.strokeStyle = rgba(S.tone.gold, 1);
      ctx.lineWidth = 2.2 * px;
      ctx.stroke();
      ctx.beginPath();
      hexPath(ctx, p.x, p.y, HEX_SIZE * 0.66);
      ctx.lineWidth = 1.4 * px;
      ctx.stroke();
    }
    ctx.restore();
  }

  // A tile-chosen build being placed (Civ's district placement): every eligible tile lit, under the
  // counters and banners; the hovered tile's heavy ring is drawn above them, further down.
  if (inp.place) {
    const pl = inp.place;
    if (pl.prices) {
      // Buying a tile: dim everything but the buyable tiles, which stay lit; the unaffordable ones read dimmer.
      ctx.save();
      ctx.beginPath();
      ctx.rect(-1e5, -1e5, 2e5, 2e5);
      for (const t of pl.prices) { const p = hxCenter(t.tile); hexPath(ctx, p.x, p.y, HEX_SIZE * 0.9); }
      ctx.fillStyle = rgba(S.tone.ink, 0.22);
      ctx.fill('evenodd');
      ctx.restore();
      drawTileCandidates(ctx, pl.prices.filter((t) => !t.affordable).map((t) => t.tile), null, S.tone.ink, px, 0, 1.4);
    }
    drawTileCandidates(ctx, pl.candidates, null, S.tone.gold, px, 0.3, 4.2);
  }

  // Ranged attack range.
  if (inp.rangedTiles) {
    ctx.beginPath();
    for (const hx of inp.rangedTiles.valid) {
      const p = hxCenter(hx);
      hexPath(ctx, p.x, p.y, HEX_SIZE * 0.96);
    }
    ctx.fillStyle = rgba(S.ink.attack, 0.16);
    ctx.fill();
    ctx.strokeStyle = rgba(S.ink.attack, 0.6);
    ctx.lineWidth = 1.5 * px;
    ctx.stroke();
    ctx.beginPath();
    for (const hx of inp.rangedTiles.blocked) {
      const p = hxCenter(hx);
      hexPath(ctx, p.x, p.y, HEX_SIZE * 0.96);
    }
    ctx.fillStyle = S.tone.wash;
    ctx.fill();
    ctx.strokeStyle = S.tone.attackWash;
    ctx.setLineDash([4 * px, 4 * px]);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // Great Migration: released tiles in a translucent red wash with a dashed edge, the shrinking cities
  // in a heavier red ring (legatum-ui: special actions show where they act; overlays stay translucent).
  if (inp.goalPreview) {
    if (inp.goalPreview.released.length) {
      ctx.beginPath();
      for (const hx of inp.goalPreview.released) {
        const p = hxCenter(hx);
        hexPath(ctx, p.x, p.y, HEX_SIZE * 0.92);
      }
      ctx.fillStyle = rgba(S.tone.bad, 0.2);
      ctx.fill();
      ctx.strokeStyle = rgba(S.tone.bad, 0.7);
      ctx.lineWidth = 1.5 * px;
      ctx.setLineDash([4 * px, 3 * px]);
      ctx.stroke();
      ctx.setLineDash([]);
    }
    if (inp.goalPreview.cities.length && !inp.goalPreview.camps.length) {
      ctx.beginPath();
      for (const hx of inp.goalPreview.cities) {
        const p = hxCenter(hx);
        hexPath(ctx, p.x, p.y, HEX_SIZE * 1.02);
      }
      ctx.strokeStyle = rgba(S.tone.bad, 0.95);
      ctx.lineWidth = 3 * px;
      ctx.stroke();
    }
    // Call the warbands: each camp it would raise (dashed blood-red hex, like a camp) joined to its city by a thin line.
    if (inp.goalPreview.camps.length) {
      ctx.beginPath();
      for (const hx of inp.goalPreview.camps) {
        const p = hxCenter(hx);
        hexPath(ctx, p.x, p.y, HEX_SIZE * 0.9);
      }
      ctx.fillStyle = rgba(S.barb.ink, 0.22);
      ctx.fill();
      ctx.setLineDash([5 * px, 3 * px]);
      ctx.strokeStyle = rgba(S.barb.blood, 0.95);
      ctx.lineWidth = 2.2 * px;
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.beginPath();
      inp.goalPreview.cities.forEach((city, i) => {
        const camp = inp.goalPreview!.camps[i];
        if (!camp) return;
        const a = hxCenter(city), b = hxCenter(camp);
        ctx.moveTo(a.x, a.y);
        ctx.lineTo(b.x, b.y);
      });
      ctx.strokeStyle = rgba(S.barb.blood, 0.55);
      ctx.lineWidth = 1.5 * px;
      ctx.setLineDash([2 * px, 4 * px]);
      ctx.stroke();
      ctx.setLineDash([]);
    }
    // A one-time unit action (retiring a veteran): the cities where it is valid, in a gold ring over a translucent wash.
    if (inp.goalPreview.places.length) {
      ctx.beginPath();
      for (const hx of inp.goalPreview.places) {
        const p = hxCenter(hx);
        hexPath(ctx, p.x, p.y, HEX_SIZE * 1.55);
      }
      ctx.fillStyle = rgba(S.tone.gold, 0.2);
      ctx.fill();
      ctx.strokeStyle = rgba(S.tone.gold, 0.98);
      ctx.lineWidth = 4 * px;
      ctx.setLineDash([9 * px, 5 * px]);
      ctx.stroke();
      ctx.setLineDash([]);
    }
  }

  // Barbarian camp tiles: a faint ink wash inside a dashed accent-red hex, so the camp reads even
  // with its guard standing on it (faded where only remembered).
  for (const camp of view.barbarian_camps ?? []) {
    const p = hxCenter(camp.pos);
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.9);
    ctx.fillStyle = rgba(S.barb.ink, camp.visible ? 0.1 : 0.06);
    ctx.fill();
    ctx.setLineDash([5 * px, 3 * px]);
    ctx.strokeStyle = rgba(S.barb.blood, camp.visible ? 0.9 : 0.5);
    ctx.lineWidth = S.pen.line * px;
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // Hover.
  if (inp.hover && tileIndex(inp.hover, W, H) >= 0) {
    const p = hxCenter(inp.hover);
    ctx.beginPath();
    hexPath(ctx, p.x, p.y, HEX_SIZE * 0.94);
    ctx.strokeStyle = S.ink.hover;
    ctx.lineWidth = 1.6 * px;
    ctx.stroke();
    if (inp.hoverSelect) {
      // Select affordance: a ring in the player's colour around what a click would select.
      ctx.beginPath();
      ctx.arc(p.x, p.y, HEX_SIZE * 0.66, 0, Math.PI * 2);
      ctx.strokeStyle = mapPlayerColor(view.me);
      ctx.lineWidth = 2.5 * px;
      ctx.stroke();
    }
  }

  // Proposed city: a dashed ring around where a settler will found.
  if (ownSel?.kind === 'Settler') {
    const target = inp.pendingMarks.get(ownSel.id) === 'found' ? ownSel.pos : inp.path?.length ? inp.path[inp.path.length - 1].hex : null;
    const ti = target ? tileIndex(target, W, H) : -1;
    if (ti >= 0) {
      const set = new Set<number>([ti]);
      for (let e = 0; e < 6; e++) {
        const nb = neighborIndex(ti, e, W, H);
        if (nb >= 0) set.add(nb);
      }
      const col = mapPlayerColor(view.me);
      ctx.beginPath();
      tracePolylines(ctx, boundary(set, (i) => set.has(i), W, H));
      ctx.fillStyle = rgba(col, 0.1);
      ctx.fill();
      ctx.strokeStyle = col;
      ctx.lineWidth = 1.6 * px;
      ctx.lineCap = 'round';
      ctx.setLineDash([6 * px, 3 * px]);
      ctx.stroke();
      ctx.setLineDash([]);
    }
  }

  // ---- screen space ----
  screenT();

  // Roads, then the Courier Hall's rhumb lines and compass-rose plaque: above terrain, below banners and units.
  if (reveal?.roads) drawRevealed(ctx, dpr, w, h, reveal.roads, view, (v) => drawRoads(ctx, v, S2, hexPx, w, h));
  else drawRoads(ctx, view, S2, hexPx, w, h);
  drawHallLand(ctx, view, S2, hexPx, w, h);
  // A closed country (Sakoku): its palisade border.
  drawClosedLand(ctx, view, S2, hexPx, w, h);

  // Vignette.
  if (S.vignette > 0) {
    ctx.save();
    ctx.translate(w / 2, h * 0.52);
    ctx.scale(w, h);
    const gr = ctx.createRadialGradient(0, 0, 0, 0, 0, 0.72);
    gr.addColorStop(0.55, rgba(S.vignetteColor, 0));
    gr.addColorStop(1, rgba(S.vignetteColor, S.vignette));
    ctx.fillStyle = gr;
    ctx.fillRect(-0.5, -0.52, 1, 1);
    ctx.restore();
  }

  // Lettering: seas, ranges and regions. Names never print over one another (seas first, then
  // ranges, then regions), and zoomed out they keep clear of the city seals. The choice depends on
  // zoom only, not on the pan, so a name never flickers as the map scrolls.
  const labelRects: Rect[] = [];
  const lscale = Math.max(0.8, Math.min(1.35, Math.sqrt(z / 0.9)));
  if (!inp.lens || inp.lens === 'default') {
    const rank = { sea: 0, range: 1, land: 2 } as const;
    const kept: Rect[] = [];
    const clear = (r: Rect, others: readonly Rect[]) => others.every((o) => r.x + r.w + 4 <= o.x || o.x + o.w + 4 <= r.x || r.y + r.h + 2 <= o.y || o.y + o.h + 2 <= r.y);
    const avoid = isFar(hexPx) ? banners.map((b) => b.cover) : [];
    for (const l of [...labelsFor(view)].sort((a, b) => rank[a.kind] - rank[b.kind])) {
      const p = worldToScreen(cam, w, h, l.x, l.y);
      const color = l.kind === 'sea' ? S.ink.sea : S.ink.land;
      const rect = drawLabel(ctx, look, l.kind, l.text, color, p.x, p.y, l.angle, lscale, dpr, false);
      if (!clear(rect, kept) || !clear(rect, avoid)) continue;
      kept.push(rect);
      if (p.x < -300 || p.x > w + 300 || p.y < -60 || p.y > h + 60) continue;
      labelRects.push(drawLabel(ctx, look, l.kind, l.text, color, p.x, p.y, l.angle, lscale, dpr));
    }
  }

  // Pending move lines for other units.
  for (const l of inp.pendingLines) {
    const a = S2(l.from);
    const b = S2(l.to);
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.strokeStyle = rgba(l.attack ? S.ink.attack : S.ink.route, 0.6);
    ctx.lineWidth = 1.5;
    ctx.setLineDash([3, 5]);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.beginPath();
    ctx.arc(b.x, b.y, 3.5, 0, Math.PI * 2);
    ctx.fillStyle = l.attack ? S.ink.attack : S.ink.route;
    ctx.fill();
  }

  // Painted ruins sit on the land beneath counters and banners; remembered art is authored at 60%.
  for (let i = 0; i < view.tiles.length; i++) {
    const t = view.tiles[i];
    if (!t.ruin || !t.explored) continue;
    const p = worldToScreen(cam, w, h, centerX(i, W), centerY(i, W));
    if (p.x < -40 || p.x > w + 40 || p.y < -40 || p.y > h + 40) continue;
    const image = mapFeatureImage('ruin', isFar(hexPx), t.visible, hexPx);
    if (image) {
      const r = isFar(hexPx) ? Math.max(4, hexPx * 0.3) : Math.max(9, hexPx * 0.88);
      ctx.drawImage(image, p.x - r, p.y - r, r * 2, r * 2);
    }
  }

  // Path preview with turn markers.
  const routeBox = inp.path && selUnit ? drawRoute(ctx, inp, S, look, S2, selUnit) : null;
  if (inp.path && inp.pathRisk) drawRouteRisk(ctx, S, S2(inp.pathRisk), hexPx);
  if (inp.pathUnreachable) {
    const p = S2(inp.pathUnreachable);
    ctx.strokeStyle = S.ink.attack;
    ctx.lineWidth = 2.5;
    ctx.lineCap = 'round';
    const r = Math.max(5, hexPx * 0.22);
    ctx.beginPath();
    ctx.moveTo(p.x - r, p.y - r);
    ctx.lineTo(p.x + r, p.y + r);
    ctx.moveTo(p.x + r, p.y - r);
    ctx.lineTo(p.x - r, p.y + r);
    ctx.stroke();
  }

  // Selection mark under the token (unit or hangar aircraft), else city.
  const pulse = 0.75 + 0.25 * Math.sin((inp.pulseTime ?? inp.time) / 300);
  let ring: SelectionRing | null = null;
  const ringAt = (x: number, y: number): void => { ring = { x, y, r: selectionRadius(hexPx) + 3.5 + 1.6 }; };
  // Zoomed in, a token behind a city banner is raised (bannerLift): its selection mark goes with it.
  const liftCovers = banners.filter((b) => !b.far).map((b) => b.cover);
  const cityHexes = new Set(view.cities.map((c) => `${c.pos.q},${c.pos.r}`));
  const liftOf = (u: UnitView, x: number, y: number) => inp.unitPos.has(u.id) || cityHexes.has(`${u.pos.q},${u.pos.r}`) ? 0 : bannerLift(x, y, tokenHalf(hexPx), hexPx, liftCovers);
  if (selUnit) {
    const wp = inp.unitPos.get(selUnit.id) ?? hxCenter(selUnit.pos);
    const p = worldToScreen(cam, w, h, wp.x, wp.y);
    if (!isFar(hexPx)) p.y -= liftOf(selUnit, p.x, p.y);
    drawSelection(ctx, look, S, p.x, p.y, hexPx, pulse);
    ringAt(p.x, p.y);
  } else if (inp.selectedAir != null && !inp.fx?.plan.shots.some((s) => s.shooter === inp.selectedAir && aircraftPassProgress(s, inp.fx!.t) != null)) {
    const plane = view.aircraft?.find((a) => a.id === inp.selectedAir);
    const home = plane && view.cities.find((c) => c.id === plane.city);
    if (home) { const p = S2(home.pos); drawSelection(ctx, look, S, p.x, p.y, hexPx, pulse); ringAt(p.x, p.y); }
  } else if (selCity) {
    const p = S2(selCity.pos);
    drawSelection(ctx, look, S, p.x, p.y, hexPx, pulse);
    ringAt(p.x, p.y);
  }

  // Painted camps stay centred under the unit counter; their clear middle is counter-sized.
  for (const camp of view.barbarian_camps ?? []) {
    const p = S2(camp.pos);
    if (p.x < -80 || p.x > w + 80 || p.y < -80 || p.y > h + 80) continue;
    const far = isFar(hexPx);
    const image = mapFeatureImage('camp', far, camp.visible, hexPx);
    if (image) {
      const r = far ? Math.max(4, hexPx * 0.3) : Math.max(9, hexPx * 0.88);
      ctx.drawImage(image, p.x - r, p.y - r, r * 2, r * 2);
    }
  }

  // Scenario objective: every target city is ringed from the first turn, fog or not.
  for (const t of inp.scenarioTargets ?? []) {
    const p = S2(t.pos);
    if (p.x < -120 || p.x > w + 120 || p.y < -120 || p.y > h + 120) continue;
    drawTargetMark(ctx, look, S, t, p.x, p.y, hexPx, pulse, !view.cities.some((c) => c.id === t.city));
  }

  // A damaged city burns for the whole turn: smoke and fires over its art, under banners and units.
  // Each box is what a clipped ambient frame repaints (MapCanvas: smoke and siege rings move without a full redraw).
  const ambient: Rect[] = [];
  // The route's dashes march on the pulse clock, so selection-pulse frames repaint its box too.
  if (routeBox) ambient.push(routeBox);
  if (inp.ambientT != null) {
    for (const c of view.cities) {
      if (!smoulderVisible(c, cityHp(c), cam, w, h)) continue;
      const damage = 1 - cityHp(c) / c.max_hp;
      const cp = S2(c.pos);
      drawSiegeAmbient(ctx, cp, damage, c.id * 97 + 5, inp.ambientT, hexPx);
      ambient.push(siegeAmbientBox(cp, hexPx));
    }
  }

  // Zoomed out a city is a seal with its name beside it (the name dropped only where it has no clear room):
  // its units sit at the seal's lower right, over it. Zoomed in, every banner waits until after the units
  // (as in Civ 6), so a token on the hex above never hides the city's name or meters.
  for (const { c, x, far, alpha: a0 } of banners) {
    if (!far) continue;
    const motion = bannerMotion?.get(c.id);
    const alpha = a0 * (motion?.alpha ?? 1);
    if (alpha <= 0.01) continue;
    if (motion && motion.sx !== 1) { ctx.save(); ctx.translate(x, 0); ctx.scale(motion.sx, 1); ctx.translate(-x, 0); }
    drawSprite(ctx, far.seal, far.x, far.y + (motion?.dy ?? 0), alpha);
    if (motion && motion.sx !== 1) ctx.restore();
    if (siege.has(c.id)) drawSiegeBar(ctx, far.x, far.y - far.seal.ay + far.seal.h + 2, Math.max(30, far.seal.w), c.hp, c.max_hp, FONTS[look].sans, false);
  }

  drawHallBadge(ctx, view, S2, hexPx, w, h);

  // Units. Sharing a tile, they fan out side by side (military right, civilian left) so both
  // tokens read; three or more also carry a count badge.
  const selectedPlane = inp.selectedAir == null ? null : view.aircraft?.find((a) => a.id === inp.selectedAir);
  const selectedAirCity = selectedPlane && view.cities.find((c) => c.id === selectedPlane.city);
  const inPass = selectedPlane && inp.fx?.plan.shots.some((s) => s.shooter === selectedPlane.id && aircraftPassProgress(s, inp.fx!.t) != null);
  const airUnit = selectedPlane && selectedAirCity && !inPass ? aircraftAsUnit(selectedPlane, selectedAirCity) : null;
  const byTile = new Map<string, UnitView[]>();
  const displayUnits = [...view.units, ...(airUnit ? [airUnit] : [])];
  for (const u of displayUnits) {
    const k = `${u.pos.q},${u.pos.r}`;
    byTile.set(k, [...(byTile.get(k) ?? []), u]);
  }
  const cityTiles = new Set(view.cities.map((c) => `${c.pos.q},${c.pos.r}`));
  const far = isFar(hexPx);
  const tw = far ? PIP_HALF : tokenHalf(hexPx);
  const unitBottoms = new Map<number, number>();
  const liftedHits: { hex: Hex; rect: Rect }[] = [];
  // A held garrison stands in its city, under the banner: its padlock is drawn after the banners so it is never hidden.
  const lockMarks: { x: number; y: number; w: number }[] = [];
  const order = [...displayUnits].sort((a, b) => a.pos.r - b.pos.r || Number(a.id === inp.selectedUnit || a.id === inp.selectedAir) - Number(b.id === inp.selectedUnit || b.id === inp.selectedAir));
  const strengthCue: StrengthCue | null = ownSel && ownSel.military && !inp.unitPos.size
    ? { sel: ownSel, selFaction: factionOf.get(ownSel.owner), targets: new Set(attackableEnemies(view, ownSel).map((t) => t.id)), neutralBarbarians: neutralBarbarianCue(view, BARBARIAN) }
    : null;
  if (inp.march?.size && !far) drawMarchDust(ctx, inp.march, (p) => worldToScreen(cam, w, h, p.x, p.y), tw, z, dpr);
  for (const unit of order) {
    const fxHp = fxNow ? hpAt(fxNow.plan, `u${unit.id}`, fxNow.t) : undefined;
    const u = fxHp !== undefined && fxHp !== unit.hp ? { ...unit, hp: fxHp } : unit;
    const wp = inp.unitPos.get(u.id) ?? hxCenter(u.pos);
    let { x, y } = worldToScreen(cam, w, h, wp.x, wp.y);
    if (x < -60 || x > w + 60 || y < -60 || y > h + 60) continue;
    // The hex's own centre and half-width, before a stack spreads its units: a covered unit's siege gear stays inside them.
    const hexBox = { cx: x, half: hexPx * (Math.sqrt(3) / 2) };
    const k = `${u.pos.q},${u.pos.r}`;
    const stack = byTile.get(k) ?? [u];
    if (stack.length > 1 && !inp.unitPos.has(u.id)) {
      const sorted = [...stack].sort((a, b) => Number(b.military) - Number(a.military) || a.id - b.id);
      const i = sorted.findIndex((s) => s.id === u.id);
      const spread = Math.min(2, stack.length);
      x += (spread === 2 ? (i === 0 ? 1 : -1) : 0) * tw * 1.16;
      y += (i === 0 ? -1 : 1) * tw * STACK_LIFT;
    }
    // Zoomed out a city's own seal fills its hex, so its units sit at the seal's lower right.
    if (cityTiles.has(k)) { if (far) { x += sealRadius(hexPx) * 0.8; y += sealRadius(hexPx) * 0.7; } else y += hexPx * CITY_UNIT_DROP; }
    if (!far) {
      const lift = liftOf(u, x, y);
      if (lift > 0) { y -= lift; liftedHits.push({ hex: u.pos, rect: { x: x - tw, y: y - tw * 1.3, w: tw * 2, h: tw * 2.6 } }); }
    }
    const shift = inp.unitShift?.get(u.id);
    if (shift) { x += shift.x * z; y += shift.y * z; }
    const stride = inp.march?.get(u.id);
    if (stride) y += stride.lift * z;
    const tile = tileIndex(u.pos, W, H);
    unitBottoms.set(tile, Math.max(unitBottoms.get(tile) ?? 0, y - S2(u.pos).y + tw + 4));
    // Byzantium's Heir is public wherever he stands: on a tile out of sight he wears a gilt halo.
    if (u.heir_of != null && !view.tiles[tile]?.visible) drawHeirHalo(ctx, x, y, tw);
    if (far) {
      drawPip(ctx, look, u.owner, u.military, x, y, tw, dpr, u.heir_of != null, u.kind as UnitKind, factionOf.get(u.owner));
      const rank = u.military ? rankLevel(u.level ?? 0, factionOf.get(u.owner)) : null;
      if (rank) {
        if (rank >= 5 && u.heir_of == null) {
          // A gilt rim remains the far-tier cue for a veteran; the badge identifies the exact level.
          ctx.beginPath(); ctx.arc(x, y, tw + 2, 0, Math.PI * 2);
          ctx.strokeStyle = MAP_STYLES[look].tone.ink; ctx.lineWidth = 4; ctx.stroke();
          ctx.strokeStyle = MAP_STYLES[look].tone.chevron; ctx.lineWidth = 2.4; ctx.stroke();
        }
        const width = 20;
        drawRankInsignia(ctx, rank, x + tw * 0.9 - width / 2, y - tw * 0.8 - width * 34 / 48, width);
      }
      if (u.hp < 100) drawTokenHealth(ctx, look, u.hp, x, y, tw);
      if (u.occupation_lock != null) lockMarks.push({ x, y, w: tw });
      const farHeir = heirTag(view, u);
      if (farHeir) drawTag(ctx, look, x, y + tw + (u.hp < 100 ? 9 : 4), farHeir, u.owner === view.me ? 'good' : u.owner === u.heir_of ? 'neutral' : 'bad');
      const farLeft = contractLeft(u, view);
      if (farLeft !== null) drawContractPip(ctx, look, x, y, tw, farLeft);
      // Far zoom keeps only the strength cue's direction: a small rising or falling mark beside the roundel.
      if (strengthCue?.targets.has(u.id)) {
        const theirs = matchupStrength(u, factionOf.get(u.owner), strengthCue.sel, false), yours = matchupStrength(strengthCue.sel, strengthCue.selFaction, u, true);
        drawStrengthChip(ctx, theirs, yours, x + tw + 6, y - tw, FONTS[look].sans, 0.8, true, strengthCue.neutralBarbarians && isBarbarian(u.owner));
      }
      continue;
    }
    if (u.occupation_lock != null) lockMarks.push({ x, y, w: tw });
    drawUnit(ctx, look, u, x, y, tw, dpr, u.owner === view.me, inp.pendingMarks.get(u.id), u.origin ?? factionOf.get(u.owner), strengthCue, siegePropFor(view, u), hexBox, contractLeft(u, view), heirTag(view, u));
    if (stack.length > 2 && stack[0].id === u.id) drawBadge(ctx, look, x - tw, y - tw, tw * 0.45, String(stack.length), true);
  }

  // Units that fell this turn stand (from the view before the turn) until their blow lands, then sink and fade.
  for (const g of inp.ghosts ?? []) {
    let { x, y } = worldToScreen(cam, w, h, g.x, g.y);
    if (x < -60 || x > w + 60 || y < -60 || y > h + 60 || g.alpha <= 0) continue;
    if (cityTiles.has(`${g.unit.pos.q},${g.unit.pos.r}`) && !far) y += hexPx * CITY_UNIT_DROP;
    y += g.sink * z;
    ctx.save();
    ctx.globalAlpha = g.alpha;
    // It topples about its feet.
    if (g.tilt) { const fy = y + tw; ctx.translate(x, fy); ctx.rotate(g.tilt); ctx.translate(-x, -fy); }
    if (far) drawPip(ctx, look, g.unit.owner, g.unit.military, x, y, tw, dpr, false, g.unit.kind as UnitKind, factionOf.get(g.unit.owner));
    else if (isBarbarian(g.unit.owner)) drawBarbarianToken(ctx, look, g.unit.kind as UnitKind, g.unit.military, x, y, tw, dpr);
    else drawToken(ctx, look, g.unit.kind as UnitKind, mapPlayerColor(g.unit.owner), g.unit.military, x, y, tw, dpr, g.unit.owner, 1, factionOf.get(g.unit.owner));
    ctx.restore();
  }

  // Zoomed out, the names go over the units, so a stack never hides a city's name.
  for (const { far: city, alpha } of banners) if (city?.name) drawSprite(ctx, city.name.sprite, city.name.x, city.name.y, alpha);
  // Zoomed out, an automated city wears an "Auto" tag above its seal (zoomed in, the banner's meter strip carries it).
  for (const { c, far } of banners) {
    if (far && focusOf(c)) drawTag(ctx, look, far.x, far.y - far.seal.ay - 15, 'Auto', 'neutral');
  }

  // Zoomed in, every banner above the units; a city under attack also wears its alarm: a ring
  // (steady under reduced motion) and a siege badge, on top of the banner or seal at both zoom levels.
  for (const { c, spec, x, y, fs, alpha, meters, cover, far: city } of banners) {
    const level = siege.get(c.id);
    if (city) {
      if (!level) continue;
      const rad = sealRadius(hexPx);
      drawSiegeSealRing(ctx, city.x, city.y, rad, level, inp.pulseTime ?? inp.time, inp.reducedMotion);
      ambient.push(siegeSealRingBox(city.x, city.y, rad));
      drawSiegeBadge(ctx, city.x + rad * 0.85, city.y - rad * 0.85, 8, dpr);
    } else {
      const motion = bannerMotion?.get(c.id);
      const drawAlpha = alpha * (motion?.alpha ?? 1);
      if (drawAlpha <= 0.01) continue;
      if (motion && motion.sx !== 1) { ctx.save(); ctx.translate(x, 0); ctx.scale(motion.sx, 1); ctx.translate(-x, 0); }
      const r = drawBanner(ctx, look, spec, x, y + (motion?.dy ?? 0), fs, dpr, drawAlpha);
      const hpNow = cityHp(c);
      if (level) {
        const barY = r.y + r.h + SIEGE_BAR_GAP;
        drawSiegeBar(ctx, x, barY, Math.min(Math.max(r.w - 8, 64), 128), hpNow, c.max_hp, FONTS[look].sans);
        if (meters.length) drawBannerMeters(ctx, look, meters, x, barY + SIEGE_BAR_H + 4, dpr);
        drawSiegeRing(ctx, cover, level, inp.pulseTime ?? inp.time, inp.reducedMotion);
        ambient.push(siegeRingBox(cover));
        drawSiegeBadge(ctx, cover.x + cover.w, cover.y, 11, dpr);
      } else {
        if (meters.length) drawBannerMeters(ctx, look, meters, x, r.y + r.h + (c.visible && hpNow < c.max_hp ? 6 : 1), dpr);
        if (c.visible && hpNow < c.max_hp) {
          const f = Math.max(0, hpNow / c.max_hp);
          const bw = Math.min(r.w - 16, 72), bx = x - bw / 2, by = r.y + r.h + 1;
          ctx.fillStyle = TOKEN_INK.atlas.paper; ctx.fillRect(bx, by, bw, 4);
          ctx.fillStyle = healthColor(f); ctx.fillRect(bx, by, bw * f, 4);
          ctx.strokeStyle = TOKEN_INK.atlas.ink; ctx.lineWidth = 0.8; ctx.strokeRect(bx, by, bw, 4);
        }
      }
      if (motion && motion.sx !== 1) ctx.restore();
    }
  }
  for (const m of lockMarks) drawLockMark(ctx, look, m.x, m.y, m.w);

  // Air-wing furniture stays on the banner edge, after the banners.
  for (const b of banners) {
    const wing = (view.aircraft ?? []).filter((a) => a.city === b.c.id && a.owner === view.me);
    if (!wing.length) continue;
    // An original capital's star seal hangs off the banner's right end: the wing badge sits left of it.
    const x = b.cover.x + b.cover.w - 2 - (b.spec.capital && !b.far ? 2 * capitalSealRadius(b.fs) + 2 : 0), y = b.cover.y - 4;
    ctx.save();
    ctx.fillStyle = TOKEN_INK.atlas.paper;
    ctx.strokeStyle = inp.seatColor;
    ctx.lineWidth = 1.5;
    ctx.beginPath(); ctx.arc(x, y, 12, 0, Math.PI * 2); ctx.fill(); ctx.stroke();
    const icon = tintedIcon(wing.some((a) => isFighter(a.kind)) ? 'unit-fighter' : 'unit-bomber', TOKEN_INK.atlas.ink, Math.ceil(15 * dpr));
    if (icon) ctx.drawImage(icon, x - 9.5, y - 7.5, 15, 15);
    // The count on its own parchment chip, so it never straddles the disc's rim.
    ctx.beginPath(); ctx.arc(x + 9, y + 9, 6, 0, Math.PI * 2); ctx.fill(); ctx.strokeStyle = TOKEN_INK.atlas.ink; ctx.lineWidth = 1; ctx.stroke();
    ctx.fillStyle = TOKEN_INK.atlas.ink;
    ctx.font = `bold 9px ${FONTS[look].sans}`;
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(String(wing.length), x + 9, y + 9.5);
    ctx.restore();
  }

  if (inp.borderTiles?.length) {
    // Owner numbers keep off tiles that hold a token or a city (they carry their own) or a resource medallion (its ring is there). The square badge is not a unit's round owner roundel.
    const busy = new Set([...unitBottoms.keys(), ...view.cities.map((c) => tileIndex(c.pos, W, H)), ...view.tiles.flatMap((t, i) => t.resource && t.explored ? [i] : [])]);
    drawOwnerOverlay(ctx, inp.borderTiles, cam, w, h, W, dpr, look, 'labels', ownerWashScale(inp.lens), busy);
  }
  if (inp.lensTiles?.length) {
    const medallions = resourceDetail(hexPx) === 'medallions';
    const cityAt = new Set(view.cities.map((c) => tileIndex(c.pos, W, H)));
    const unitsAt = new Map<number, number>();
    for (const u of view.units) { const t = tileIndex(u.pos, W, H); unitsAt.set(t, (unitsAt.get(t) ?? 0) + 1); }
    drawLens(ctx, inp.lensTiles, cam, w, h, W, dpr, look, 'labels', unitBottoms, [...(inp.lensAvoid ?? []), ...bannerCovers],
      (i) => medallions && view.tiles[i]?.resource && view.tiles[i]?.explored ? medallionTop(hexPx, cityAt.has(i), (unitsAt.get(i) ?? 0) > 1) : 0, { yieldsOn: overlays.yields, yieldRects: overlays.yields && inp.lensTiles.some((t) => t.rank) ? yieldClusterRects(inp.yieldTiles ?? [], cam, w, h, W) : [] });
  }

  if (inp.reportArrow && inp.reportArrow.alpha > 0) {
    const r = inp.reportArrow;
    const b = S2(r.to);
    const ink = S.ink.attack, halo = S.tone.vellum;
    ctx.save();
    ctx.globalAlpha = r.alpha;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    const stroke = (path: () => void, width: number) => {
      for (const [color, w] of [[halo, width + 3], [ink, width]] as const) {
        ctx.beginPath(); path(); ctx.strokeStyle = color; ctx.lineWidth = w; ctx.stroke();
      }
    };
    const ring = Math.max(8, hexPx * 0.62);
    stroke(() => ctx.arc(b.x, b.y, ring, 0, Math.PI * 2), 2.5);
    if (r.from) {
      const a = S2(r.from);
      const len = Math.hypot(b.x - a.x, b.y - a.y);
      if (len > 1) {
        const ux = (b.x - a.x) / len, uy = (b.y - a.y) / len;
        const start = { x: a.x + ux * hexPx * 0.3, y: a.y + uy * hexPx * 0.3 };
        const tip = { x: b.x - ux * ring, y: b.y - uy * ring };
        const head = Math.max(8, Math.min(16, hexPx * 0.4));
        const side = (k: number) => ({ x: tip.x - ux * head - uy * head * 0.55 * k, y: tip.y - uy * head + ux * head * 0.55 * k });
        stroke(() => { ctx.moveTo(start.x, start.y); ctx.lineTo(tip.x - ux * head * 0.6, tip.y - uy * head * 0.6); }, 3.5);
        stroke(() => ctx.arc(start.x, start.y, 3.5, 0, Math.PI * 2), 2);
        const [l, rr] = [side(1), side(-1)];
        ctx.beginPath(); ctx.moveTo(tip.x, tip.y); ctx.lineTo(l.x, l.y); ctx.lineTo(rr.x, rr.y); ctx.closePath();
        ctx.strokeStyle = halo; ctx.lineWidth = 3; ctx.stroke();
        ctx.fillStyle = ink; ctx.fill();
      }
    }
    ctx.restore();
  }

  // Combat playback: missiles, clashes, losses and captures over the units and banners, under the damage cues.
  if (inp.fx) {
    const f0 = performance.now();
    drawCombatFx(ctx, inp.fx.plan, inp.fx.t, { toScreen: (p) => worldToScreen(cam, w, h, p.x, p.y), hexPx, far, w, h, colorOf: mapPlayerColor }, (d) => deathPos(inp.fx!.plan, d));
    const fms = performance.now() - f0;
    perf.fxFrames++;
    perf.fxMs += fms;
    perf.fxMaxMs = Math.max(perf.fxMaxMs, fms);
  }
  // Payoff moments over the units and banners: survey stakes, rising buildings and wonders.
  if (moments && momentView) {
    const m0 = performance.now();
    drawMomentFx(ctx, moments.plan, moments.t, momentView);
    const mms = performance.now() - m0;
    perf.momentFrames++;
    perf.momentMs += mms;
    perf.momentMaxMs = Math.max(perf.momentMaxMs, mms);
  }

  // The hovered or clicked improvement tile: a heavy ring above the unit counters, so a unit standing
  // on the tile never hides what the picker row points at.
  if (inp.improve && (inp.improve.hover || inp.improve.pick)) {
    const own = inp.seatColor;
    worldT();
    ctx.save();
    ctx.setLineDash([]);
    for (const hx of [inp.improve.pick, inp.improve.hover]) {
      if (!hx || !inp.improve.candidates.some((c) => c.q === hx.q && c.r === hx.r)) continue;
      const p = hxCenter(hx);
      ctx.beginPath();
      hexPath(ctx, p.x, p.y, HEX_SIZE * 0.88);
      ctx.fillStyle = rgba(own, 0.12);
      ctx.fill();
      ctx.strokeStyle = rgba(own, 1);
      ctx.lineWidth = 4.5 * px;
      ctx.stroke();
    }
    ctx.restore();
    screenT();
  }

  // The placement's hovered tile, above the unit counters so a unit standing on it never hides it. Drawn before the price chips so its wash never dims its own price.
  if (inp.place?.hover) {
    worldT();
    drawTileCandidates(ctx, [inp.place.hover], inp.place.hover, S.tone.gold, px, 0.3, 4.2);
    screenT();
  }

  // Gold prices while a tile is being bought (greyed when the treasury cannot pay), and the mark on a queued purchase,
  // above the unit counters so a unit standing on the tile never hides its price.
  const chips = [...(inp.place?.prices ?? []).map((t) => ({ ...t, queued: false })), ...(inp.tileBuys ?? []).map((t) => ({ ...t, affordable: true, queued: true }))];
  if (chips.length) {
    const own = inp.seatColor;
    worldT();
    ctx.save();
    for (const b of inp.tileBuys ?? []) {
      const p = hxCenter(b.tile);
      ctx.beginPath();
      hexPath(ctx, p.x, p.y, HEX_SIZE * 0.9);
      ctx.fillStyle = rgba(own, 0.16);
      ctx.fill();
      ctx.strokeStyle = rgba(own, 1);
      ctx.lineWidth = 3 * px;
      ctx.setLineDash([]);
      ctx.stroke();
    }
    ctx.restore();
    screenT();
  }
  if (chips.length && hexPx >= 22) {
    const own = inp.seatColor;
    ctx.save();
    ctx.font = `700 ${hexPx >= 40 ? 12 : 10.5}px ${FONTS[look].sans}`;
    ctx.textBaseline = 'middle';
    for (const b of chips) {
      const c = S2(b.tile);
      const text = String(b.price);
      const tw = ctx.measureText(text).width + 19, th = 16;
      const x = c.x - tw / 2, y = c.y + hexPx * 0.42 - th / 2;
      ctx.fillStyle = b.queued ? own : rgba(S.tone.vellum, 0.96);
      ctx.fillRect(x, y, tw, th);
      ctx.strokeStyle = b.affordable ? own : rgba(S.tone.ink, 0.45);
      ctx.lineWidth = 1;
      ctx.strokeRect(x + 0.5, y + 0.5, tw - 1, th - 1);
      ctx.beginPath();
      ctx.arc(x + 8, y + th / 2, 4.2, 0, Math.PI * 2);
      ctx.fillStyle = !b.affordable ? rgba(S.tone.ink, 0.35) : S.tone.gold;
      ctx.fill();
      ctx.strokeStyle = rgba(S.tone.ink, 0.7);
      ctx.stroke();
      ctx.fillStyle = b.queued ? S.tone.vellum : b.affordable ? S.tone.ink : rgba(S.tone.ink, 0.6);
      ctx.textAlign = 'left';
      ctx.fillText(text, x + 15, y + th / 2 + 0.5);
    }
    ctx.restore();
  }

  // The aircraft picker's candidates, above the unit counters so a unit standing on a target never hides it.
  if (inp.air?.pickKind) {
    worldT();
    drawAirCandidates(ctx, inp.air, inp.air.pickKind === 'strike' ? S.ink.attack : inp.seatColor, px);
    screenT();
    drawAirOffscreen(ctx, inp.air, (x, y) => worldToScreen(cam, w, h, x, y), inp.seatColor, FONTS[look].sans, { w, h }, inp.lensAvoid);
  }

  // Battle results from the last turn: damage and losses on the hexes involved, fading out.
  for (const c of inp.cues ?? []) {
    if (c.alpha <= 0) continue;
    const p = S2(c.hex);
    ctx.save();
    ctx.globalAlpha = c.alpha;
    ctx.font = `700 ${Math.round(Math.max(12, Math.min(18, hexPx * 0.5)))}px ${FONTS[look].sans}`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.lineJoin = 'round';
    ctx.lineWidth = 4;
    ctx.strokeStyle = S.tone.vellum;
    const ty = p.y - hexPx * 0.95 - c.rise;
    ctx.strokeText(c.text, p.x, ty);
    ctx.fillStyle = c.tone === 'bad' ? S.tone.bad : c.tone === 'good' ? S.tone.good : S.tone.ink;
    ctx.fillText(c.text, p.x, ty);
    ctx.restore();
  }

  const celebration = inp.celebration;
  if (celebration?.moment.at && !celebration.reduced) {
    const { moment, elapsed, reduced } = celebration;
    const p = S2(moment.at!);
    const f = celebrationFrame(elapsed, reduced);
    const gold = moment.tier === 'notable' ? S.tone.goldNotable : S.tone.gold;
    const outline = S.tone.vellum;
    ctx.save();
    ctx.strokeStyle = gold;
    ctx.fillStyle = gold;
    ctx.lineWidth = moment.tier === 'minor' ? 2 : 3;
    ctx.globalAlpha = moment.tier === 'major' || moment.tier === 'legendary' ? Math.max(.55, f.alpha) : f.alpha;
    // A quiet gilt rule below the label is enough; radiating rings and spokes hid the city and its border.
    if (moment.scout) {
      const to = S2(moment.scout.to);
      ctx.globalAlpha = .6;
      ctx.setLineDash([4, 5]); ctx.beginPath(); ctx.moveTo(p.x, p.y); ctx.lineTo(to.x, to.y); ctx.stroke(); ctx.setLineDash([]);
      ctx.beginPath(); ctx.arc(to.x, to.y, hexPx * .7, 0, Math.PI * 2); ctx.stroke();
    }
    ctx.globalAlpha = 1;
    ctx.font = `700 15px ${FONTS[look].sans}`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.strokeStyle = outline;
    ctx.lineWidth = 4;
    const caption = moment.tier === 'minor' ? moment.effect : moment.title;
    // On a paper slip over the celebrated hex itself, below its centre: at 1.65 hexes down it sat
    // on a unit in the next row (playtest-6); the slip keeps it legible over the city art.
    const cy = p.y + hexPx * 0.5;
    const cw = ctx.measureText(caption).width + 14;
    ctx.globalAlpha = .92;
    ctx.fillStyle = outline;
    ctx.fillRect(p.x - cw / 2, cy - 11, cw, 22);
    ctx.globalAlpha = 1;
    ctx.lineWidth = 1;
    ctx.strokeStyle = gold;
    ctx.strokeRect(p.x - cw / 2 + .5, cy - 10.5, cw - 1, 21);
    ctx.fillStyle = gold;
    ctx.fillText(caption, p.x, cy + 1);
    ctx.beginPath(); ctx.moveTo(p.x - cw / 3, cy + 14); ctx.lineTo(p.x + cw / 3, cy + 14); ctx.stroke();
    ctx.restore();
  }

  const ms = performance.now() - t0;
  if (perf.measureFrames)
    performance.measure('legatum:mapDraw', { start: t0, end: t0 + ms });
  perf.frames++;
  perf.drawMs += ms;
  noteMapDraw(ms);
  perf.workMs += terrain.lastWorkMs;
  perf.maxMs = Math.max(perf.maxMs, ms);
  // City banners, so hover cards can keep clear of them (a card over your own city's banner hides it).
  // idle: whether idle life has anything in view to move (the loop stops ticking for it when not).
  return { pending, labelRects, bannerRects: banners.map((b) => b.cover),
    bannerHits: banners.map((b) => {
      const rect = b.far ? b.cover : bannerRect(look, b.spec, b.x, b.y, b.fs, dpr);
      // What is drawn: the plate, then the strip under it (meters, siege bar or HP bar), centred and no wider than they are.
      const under = Math.max(b.meters.length ? bannerMetersSize(look, b.meters, dpr).w : 0, siege.has(b.c.id) ? Math.min(Math.max(rect.w - 8, 64), 128) : 0, cityHp(b.c) < b.c.max_hp ? Math.min(rect.w - 16, 72) : 0);
      const parts = b.far ? [b.cover] : [rect, { x: b.x - under / 2, y: rect.y + rect.h, w: under, h: b.cover.h - rect.h }];
      return { city: b.c, rect, cover: b.cover, parts, far: !!b.far };
    }),
    liftedHits,
    idle: idleLive > 0, ring, ambient };
}

/** A scenario target: a double ring around the city's hex (red to take or contain, ink to hold)
 *  with the objective's mark on top: crosshair, shield or dashed ring. A city the seat hasn't
 *  seen yet is named under the ring, so the objective is findable from turn 1. */
function drawTargetMark(ctx: CanvasRenderingContext2D, look: LookId, S: MapStyle, t: ScenarioTarget, x: number, y: number, hexPx: number, pulse: number, unseen: boolean) {
  const halo = S.tone.vellum;
  const ink = t.kind === 'Hold' ? TOKEN_INK.atlas.ink : S.ink.attack;
  const r = Math.max(14, hexPx * 0.92);
  ctx.save();
  ctx.lineCap = 'round';
  for (const [color, width] of [[halo, 5], [ink, 2.2]] as const) {
    ctx.beginPath();
    ctx.arc(x, y, r, 0, Math.PI * 2);
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.setLineDash(t.kind === 'Contain' && color === ink ? [7, 4] : []);
    ctx.stroke();
  }
  ctx.setLineDash([]);
  ctx.globalAlpha = 0.35 + 0.35 * pulse;
  ctx.beginPath();
  ctx.arc(x, y, r + 5, 0, Math.PI * 2);
  ctx.strokeStyle = ink;
  ctx.lineWidth = 1.2;
  ctx.stroke();
  ctx.globalAlpha = 1;
  // The mark sits on the ring's upper right, on a halo disc.
  const bx = x + r * 0.72, by = y - r * 0.72, br = Math.max(7, Math.min(11, hexPx * 0.3));
  ctx.beginPath();
  ctx.arc(bx, by, br, 0, Math.PI * 2);
  ctx.fillStyle = halo;
  ctx.fill();
  ctx.strokeStyle = ink;
  ctx.lineWidth = 1.5;
  ctx.stroke();
  const m = br * 0.62;
  ctx.beginPath();
  if (t.kind === 'Take') {
    ctx.arc(bx, by, m * 0.62, 0, Math.PI * 2);
    ctx.moveTo(bx, by - m); ctx.lineTo(bx, by - m * 0.35);
    ctx.moveTo(bx, by + m); ctx.lineTo(bx, by + m * 0.35);
    ctx.moveTo(bx - m, by); ctx.lineTo(bx - m * 0.35, by);
    ctx.moveTo(bx + m, by); ctx.lineTo(bx + m * 0.35, by);
  } else if (t.kind === 'Hold') {
    ctx.moveTo(bx, by - m);
    ctx.lineTo(bx + m * 0.85, by - m * 0.6);
    ctx.lineTo(bx + m * 0.75, by + m * 0.2);
    ctx.quadraticCurveTo(bx + m * 0.5, by + m * 0.8, bx, by + m);
    ctx.quadraticCurveTo(bx - m * 0.5, by + m * 0.8, bx - m * 0.75, by + m * 0.2);
    ctx.lineTo(bx - m * 0.85, by - m * 0.6);
    ctx.closePath();
  } else {
    ctx.arc(bx, by, m * 0.75, 0, Math.PI * 2);
  }
  ctx.strokeStyle = ink;
  ctx.lineWidth = 1.6;
  ctx.stroke();
  if (unseen) {
    const fs = Math.round(Math.max(11, Math.min(14, hexPx * 0.36)));
    ctx.font = `600 ${fs}px ${FONTS[look].sans}`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    ctx.lineJoin = 'round';
    ctx.lineWidth = 4;
    ctx.strokeStyle = halo;
    const label = `${t.name} (not yet seen)`;
    ctx.strokeText(label, x, y + r + 4);
    ctx.fillStyle = ink;
    ctx.fillText(label, x, y + r + 4);
  }
  ctx.restore();
}

/** The icon each automatic-build focus wears on the banner. */
const FOCUS_ICON: Record<CityBuildFocus, string> = { Balanced: 'ui-random', Science: 'yield-science', Gold: 'yield-gold', Military: 'unit-spearman' };

const q24 = (f: number) => Math.round(Math.max(0, Math.min(1, f)) * 24) / 24;

/** Growth (the territory bar: each growth claims a tile) and production meters for an own city's banner, from the same forecasts as the city panel. */
export function bannerMeters(c: CityView, faction?: Faction, focus: CityBuildFocus | null = null, chosen: ProductionItem | null = null): BannerMeter[] {
  const growth = growthProgress(c);
  if (!growth) return [];
  const out: BannerMeter[] = [{
    kind: 'growth', fraction: q24(growth.needed > 0 ? growth.stored / growth.needed : 0),
    text: growth.turns !== null ? String(growth.turns) : '–', bad: growth.state === 'starving', paused: growth.state !== 'growing' && growth.state !== 'starving',
  }];
  const production = productionProgress(c);
  // Occupation lock experiment: a frozen city shows the seal of a held city, not the red Build prompt.
  if (production && c.occupation_lock != null) out.push({ kind: 'production', fraction: 0, text: 'Held', paused: true, icon: 'fortify', inkIcon: true });
  else if (production && chosen && !sameItem(chosen, c.detail?.producing ?? null)) {
    // Chosen this turn, not yet resolved: the banner shows the choice at once, with the forecast its build card gives.
    const option = c.detail?.options.find((o) => sameItem(o.item, chosen));
    out.push({
      kind: 'production', fraction: q24(option && option.cost > 0 ? option.production_stored / option.cost : 0),
      // No output (or a project with no turn count) has no forecast: a dash, as the build card prints no number.
      text: option && option.turns > 0 ? String(option.turns) : '–', icon: itemIcon(chosen, faction),
    });
  } else if (production) {
    const auto = focus ? { label: 'Auto', icon: FOCUS_ICON[focus] } : undefined;
    out.push(production.state === 'idle'
      // An automated idle city chooses at Ready: it shows its focus, not the red "Build" prompt.
      ? auto ? { kind: 'production', fraction: 0, text: 'Auto', paused: true, icon: auto.icon, inkIcon: true } : { kind: 'production', fraction: 0, text: 'Build', bad: true, icon: null }
      : {
        kind: 'production', fraction: q24(production.needed > 0 ? production.stored / production.needed : 0),
        text: production.state === 'building' ? (production.turns !== null ? String(production.turns) : '–') : '–',
        bad: production.state === 'stalled', paused: production.state === 'waiting', icon: itemIcon(production.item!, faction), auto,
      });
  }
  return out;
}

/** A city zoomed out: its seal (centred on the hex) and, if there is clear room, its name beside it. */
interface FarCity { x: number; y: number; seal: ReturnType<typeof sealSprite>; name: { sprite: ReturnType<typeof nameSprite>; x: number; y: number } | null }

/** What a city's banner and far-zoom seal show. The star marks the original capital for the whole game, in the founder's ink; a replacement capital gets none. */
export function bannerSpec(view: Pick<PlayerView, 'me' | 'team' | 'players'>, c: CityView, glyph: string | null, selected: boolean): BannerSpec {
  return { owner: c.owner, name: c.name, pop: c.visible ? c.population : null, color: mapPlayerColor(c.owner), capital: capitalStar(view, c), selected, glyph, walls: c.visible && c.walls };
}

/** Every on-screen city banner: its spec, anchor and the screen rect it covers (health bar included). Zoomed out, the banner is a seal and a name instead. */
function cityBanners(view: PlayerView, cam: Camera, w: number, h: number, dpr: number, look: LookId, selectedCity: number | null, siege: ReadonlyMap<number, SiegeLevel>, focusOf: (c: CityView) => CityBuildFocus | null, unscored?: ReadonlySet<number>, builds?: ReadonlyMap<number, ProductionItem>) {
  const hexPx = HEX_SIZE * cam.zoom;
  const far = isFar(hexPx);
  const fs = Math.round(Math.max(11, Math.min(16, hexPx * 0.42)) * 2) / 2;
  const items = view.cities.flatMap((c) => {
    const p = worldToScreen(cam, w, h, hexToPixel(c.pos, HEX_SIZE).x, hexToPixel(c.pos, HEX_SIZE).y);
    if (p.x < -200 || p.x > w + 200 || p.y < -80 || p.y > h + 120) return [];
    const mine = c.owner === view.me;
    const faction = view.players.find((pl) => pl.id === c.owner)?.faction;
    // Every named banner needs a shape for ownership, including the smallest named hexes.
    const glyph = faction ? `glyph-${faction.toLowerCase()}` : null;
    // A capital taken this turn keeps its filled star until the VP seal lands (momentFx.ts), then goes hollow.
    const marked = bannerSpec(view, c, glyph, c.id === selectedCity);
    const spec = marked.capital?.scored && unscored?.has(c.id) ? { ...marked, capital: { ...marked.capital, scored: false } } : marked;
    const alpha = c.visible ? 1 : 0.85;
    if (far) {
      const r = sealRadius(hexPx);
      const seal = sealSprite(look, spec, r, dpr);
      const farCity: FarCity = { x: p.x, y: p.y - hexPx * 0.1, seal, name: null };
      const cover = { x: farCity.x - seal.ax, y: farCity.y - seal.ay, w: seal.w, h: seal.h + (siege.has(c.id) ? 10 : 0) };
      return [{ c, spec, x: p.x, y: p.y, fs, alpha, cover, meters: [] as BannerMeter[], far: farCity }];
    }
    // Own cities only (rivals' Food, borders and production are private): growth, border and
    // production meters, readable from normal zoom in, so the player can watch them without
    // opening the city.
    const meters = mine && c.visible && hexPx >= 20 ? bannerMeters(c, faction, focusOf(c), builds?.get(c.id) ?? null) : [];
    const { y, cover } = bannerBlock(c, spec, meters, p, hexPx, fs, look, dpr, siege);
    return [{ c, spec, x: p.x, y, fs, alpha, meters, cover, far: null as FarCity | null }];
  });
  // The Courier Hall's badge sits beside its city's seal: names keep clear of it.
  const badges = far ? hallSites(view).map(({ city }) => {
    const q = worldToScreen(cam, w, h, hexToPixel(city.pos, HEX_SIZE).x, hexToPixel(city.pos, HEX_SIZE).y);
    const b = hallBadgeAt(q, hexPx);
    return { x: b.x - b.r, y: b.y - b.r, w: b.r * 2, h: b.r * 2 };
  }) : [];
  if (far) placeFarNames(items, view.me, selectedCity, dpr, look, badges);
  return items;
}

/** A named banner's block (plate, meters, siege or damage bar) for a city centred at `p`, in screen px. */
function bannerBlock(c: CityView, spec: BannerSpec, meters: BannerMeter[], p: { x: number; y: number }, hexPx: number, fs: number, look: LookId, dpr: number, siege: ReadonlyMap<number, SiegeLevel>) {
  const metersH = meters.length ? bannerMetersSize(look, meters, dpr).h + 1 : 0;
  // The block (plate and meters) ends BANNER_BOTTOM above the centre, over the town's roofline: the meters are a fixed
  // 17 px, so hung under the plate they covered the top of the town, most of it at small zoom.
  const y = p.y - hexPx * BANNER_BOTTOM - metersH;
  const rect = bannerRect(look, spec, p.x, y, fs, dpr);
  return { y, cover: { ...rect, h: rect.h + (siege.has(c.id) ? SIEGE_BAR_H + SIEGE_BAR_GAP + 3 : c.visible && c.hp < c.max_hp ? 6 : 0) + metersH } };
}

/**
 * Every city's banner block and painted town as they stand at LANDMARK_REF_ZOOM, in world units × LANDMARK_REF_ZOOM: what a wonder's footprint
 * is chosen against (landmarks.ts), so the choice depends on neither the camera's zoom nor what is on screen.
 */
const refCoverMemo = new WeakMap<PlayerView, { look: LookId; siege: ReadonlyMap<number, SiegeLevel>; covers: Rect[] }>();
export function referenceBannerCovers(view: PlayerView, look: LookId, siege: ReadonlyMap<number, SiegeLevel>): Rect[] {
  const hit = refCoverMemo.get(view);
  if (hit && hit.look === look && hit.siege === siege) return hit.covers;
  const hexPx = HEX_SIZE * LANDMARK_REF_ZOOM;
  const fs = Math.round(Math.max(11, Math.min(16, hexPx * 0.42)) * 2) / 2;
  const covers = view.cities.map((c) => {
    const world = hexToPixel(c.pos, HEX_SIZE);
    const faction = view.players.find((pl) => pl.id === c.owner)?.faction;
    const spec = bannerSpec(view, c, faction ? `glyph-${faction.toLowerCase()}` : null, false);
    const meters = c.owner === view.me && c.visible ? bannerMeters(c, faction) : [];
    const at = { x: world.x * LANDMARK_REF_ZOOM, y: world.y * LANDMARK_REF_ZOOM };
    // The painted town too, so a wonder beside a city never stands on its towers (UI review, 2026-11-22).
    return [bannerBlock(c, spec, meters, at, hexPx, fs, look, 1, siege).cover, townCover(at, hexPx)];
  }).flat();
  refCoverMemo.set(view, { look, siege, covers });
  return covers;
}

/** Names beside the seals, own and capital cities first: right of the seal, else left, below, above; no name where all four would print over another seal or name. */
function placeFarNames(items: { c: CityView; cover: { x: number; y: number; w: number; h: number }; far: FarCity | null; spec: { name: string } }[], me: number, selected: number | null, dpr: number, look: LookId, keepClear: { x: number; y: number; w: number; h: number }[] = []) {
  const taken = [...items.map((i) => ({ ...i.cover })), ...keepClear];
  const own = new Map(items.map((i, k) => [i, k]));
  const order = [...items].sort((a, b) => Number(b.c.id === selected) - Number(a.c.id === selected) || Number(b.c.owner === me) - Number(a.c.owner === me)
    || Number(!!b.c.original_capital) - Number(!!a.c.original_capital) || b.c.population - a.c.population || a.c.id - b.c.id);
  const free = (r: { x: number; y: number; w: number; h: number }, self: number) => taken.every((t, k) => k === self || r.x + r.w + 2 <= t.x || t.x + t.w + 2 <= r.x || r.y + r.h + 1 <= t.y || t.y + t.h + 1 <= r.y);
  for (const item of order) {
    const f = item.far!;
    const sprite = nameSprite(look, item.spec.name, dpr);
    const rx = f.seal.w / 2 - 3, ry = f.seal.h / 2 - 3;
    const spots = [
      { x: f.x + rx + sprite.w / 2 - 1, y: f.y }, { x: f.x - rx - sprite.w / 2 + 1, y: f.y },
      { x: f.x, y: f.y + ry + sprite.h / 2 - 1 }, { x: f.x, y: f.y - ry - sprite.h / 2 + 1 },
    ];
    const self = own.get(item)!;
    const at = spots.find((s) => free({ x: s.x - sprite.w / 2, y: s.y - sprite.h / 2, w: sprite.w, h: sprite.h }, self));
    if (!at) continue;
    f.name = { sprite, x: at.x, y: at.y };
    const rect = { x: at.x - sprite.w / 2, y: at.y - sprite.h / 2, w: sprite.w, h: sprite.h };
    taken.push(rect);
    const cover = item.cover;
    const x0 = Math.min(cover.x, rect.x), y0 = Math.min(cover.y, rect.y);
    item.cover = { x: x0, y: y0, w: Math.max(cover.x + cover.w, rect.x + rect.w) - x0, h: Math.max(cover.y + cover.h, rect.y + rect.h) - y0 };
  }
}

/** Where the selection ring was drawn (screen CSS pixels), outer edge included. */
export interface SelectionRing { x: number; y: number; r: number }
const selectionRadius = (hexPx: number) => hexPx * (24 / 27) * 0.92;

/** The device-pixel box a pulse-only frame repaints: the ring's outer edge, padded for anti-aliasing, kept on the canvas. */
export function ringClipRect(ring: SelectionRing, dpr: number, pad = 2): { x: number; y: number; w: number; h: number } {
  const x0 = Math.max(0, Math.floor((ring.x - ring.r) * dpr) - pad), y0 = Math.max(0, Math.floor((ring.y - ring.r) * dpr) - pad);
  const x1 = Math.ceil((ring.x + ring.r) * dpr) + pad, y1 = Math.ceil((ring.y + ring.r) * dpr) + pad;
  return { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
}

/** Where idle life paints at `t` (CSS px), for the view as the camera shows it: what an idle tick repaints. */
export function idleClipBoxes(view: PlayerView, cam: Camera, w: number, h: number, t: number): Rect[] {
  const hexPx = HEX_SIZE * cam.zoom;
  return idleBoxes(view, t, { toScreen: (p) => worldToScreen(cam, w, h, p.x, p.y), hexPx, far: isFar(hexPx), w, h, dpr: 1 });
}

/**
 * Where the hover mark and the route preview draw, in CSS pixels: what a frame repaints when only the tile under
 * the pointer changed (MapCanvas clips to these boxes for the old and the new hover instead of redrawing the map).
 */
export function hoverClipBoxes(inp: Pick<RenderInput, 'view' | 'cam' | 'width' | 'height' | 'hover' | 'path' | 'pathRisk' | 'pathUnreachable' | 'selectedUnit' | 'pendingMarks'>): Rect[] {
  const { view, cam, width: w, height: h } = inp;
  const hexPx = HEX_SIZE * cam.zoom;
  const at = (hx: Hex) => { const p = hexToPixel(hx, HEX_SIZE); return worldToScreen(cam, w, h, p.x, p.y); };
  const around = (hx: Hex, half: number): Rect => { const p = at(hx); return { x: p.x - half, y: p.y - half, w: half * 2, h: half * 2 }; };
  const out: Rect[] = [];
  // The hover hex (radius 0.94 of a hex) and its select ring, with room for the stroke.
  if (inp.hover) out.push(around(inp.hover, hexPx + 4));
  const unit = inp.selectedUnit !== null ? view.units.find((u) => u.id === inp.selectedUnit) ?? null : null;
  if (unit && inp.path?.length) {
    // The route line, its shadow and turn roundels (radius 11), and the risk mark on one of its tiles.
    const pts = [unit.pos, ...inp.path.map((st) => st.hex)].map(at);
    const pad = Math.max(16, hexPx);
    const xs = pts.map((p) => p.x), ys = pts.map((p) => p.y);
    const x0 = Math.min(...xs) - pad, y0 = Math.min(...ys) - pad;
    out.push({ x: x0, y: y0, w: Math.max(...xs) + pad - x0, h: Math.max(...ys) + pad - y0 });
  }
  if (inp.pathRisk) out.push(around(inp.pathRisk, hexPx + 4));
  if (inp.pathUnreachable) out.push(around(inp.pathUnreachable, Math.max(12, hexPx)));
  // A Settler's proposed city: the dashed ring around the target tile and its six neighbours.
  if (unit?.kind === 'Settler' && unit.owner === view.me) {
    const target = inp.pendingMarks.get(unit.id) === 'found' ? unit.pos : inp.path?.length ? inp.path[inp.path.length - 1].hex : null;
    if (target) out.push(around(target, hexPx * 2.75 + 4));
  }
  return out;
}

/** The device-pixel boxes an ambient-only frame repaints: the selection ring's, then each smoke column's and siege ring's, kept on the canvas. */
export function ambientClipRects(ring: SelectionRing | null, ambient: readonly Rect[], dpr: number, pad = 2): { x: number; y: number; w: number; h: number }[] {
  const out = ring ? [ringClipRect(ring, dpr, pad)] : [];
  for (const r of ambient) {
    const x0 = Math.max(0, Math.floor(r.x * dpr) - pad), y0 = Math.max(0, Math.floor(r.y * dpr) - pad);
    const x1 = Math.ceil((r.x + r.w) * dpr) + pad, y1 = Math.ceil((r.y + r.h) * dpr) + pad;
    if (x1 > x0 && y1 > y0) out.push({ x: x0, y: y0, w: x1 - x0, h: y1 - y0 });
  }
  return out;
}

function drawSelection(ctx: CanvasRenderingContext2D, _look: LookId, S: MapStyle, x: number, y: number, hexPx: number, pulse: number) {
  const r = selectionRadius(hexPx);
  ctx.save();
  ctx.translate(x, y);
  ctx.strokeStyle = S.ink.select;
  ctx.globalAlpha = 0.55 + 0.45 * pulse;
  ctx.lineWidth = 1.6;
  ctx.beginPath();
  ctx.arc(0, 0, r, 0, Math.PI * 2);
  ctx.stroke();
  ctx.lineWidth = 0.8;
  ctx.beginPath();
  ctx.arc(0, 0, r + 3.5, 0, Math.PI * 2);
  ctx.stroke();
  ctx.restore();
}

/** Your selected military unit and the enemies it could attack this turn (the strength cue). */
interface StrengthCue { sel: UnitView; selFaction?: Faction; targets: Set<number>; neutralBarbarians: boolean }

/** Pending-order badges are icons (the same ones the panels use), each with a plain-text label for accessibility tooling. */
const MARK_ICON: Record<PendingMark, string> = { explore: 'unit-scout', move: 'mark-move', sleep: 'mark-sleep', heal: 'mark-heal', skip: 'mark-skip', found: 'mark-found', disband: 'ui-cross', ranged: 'unit-archer', upgrade: 'ui-trend-up', invalid: 'mark-invalid' };

/** Occupation lock experiment: a padlock on the token's top-left shoulder (the right is the order badge, the left foot the rank), so a unit a rule holds in place looks held before it is asked to move. */
function drawLockMark(ctx: CanvasRenderingContext2D, look: LookId, x: number, y: number, w: number) {
  drawBadge(ctx, look, x - w * 0.95, y - w * 0.85, w * 0.55, '', false, 'mark-lock');
}

function drawUnit(ctx: CanvasRenderingContext2D, look: LookId, u: UnitView, x: number, y: number, w: number, dpr: number, mine: boolean, mark: PendingMark | undefined, faction?: Faction, cue?: StrengthCue | null, prop?: SiegeProp | null, hexBox?: { cx: number; half: number }, contract?: number | null, heirLabel?: string | null) {
  if (isBarbarian(u.owner)) drawBarbarianToken(ctx, look, u.kind as UnitKind, u.military, x, y, w, dpr);
  else drawToken(ctx, look, u.kind as UnitKind, mapPlayerColor(u.owner), u.military, x, y, w, dpr, u.owner, 1, faction, u.heir_of == null ? undefined : { crestColor: u.heir_of === u.owner ? undefined : mapPlayerColor(u.heir_of) });
  // A unit an Engineer covers wears its siege gear in front of its feet, on the hex's right side and always inside the hex: the
  // stack already fills the hex, so the gear overlaps its own figure rather than the unit on the next hex (user, 2026-10-19).
  const gear = prop && hexBox ? siegePropImage(prop) : null;
  if (gear && prop && hexBox) {
    const r = siegePropRect(prop, w, hexBox.cx, hexBox.half, y);
    ctx.drawImage(gear, r.x, r.y, r.size, r.size);
  }
  // A painted miniature stands lower (its base is bigger), so its health bar and tag sit further down.
  const mini = isBarbarian(u.owner) ? barbarianMiniature(u.kind as UnitKind) : !!miniatureImage(u.kind as UnitKind, faction);
  // The era plaque on the front of a military miniature's base (user, 2026-10-11: tell the eras apart); the health bar drops under it.
  let drop = mini ? w * 0.8 : 0;
  if (mini && u.military) {
    const g = miniatureGeometry(w);
    const py = y + g.baseDy + g.baseRy * 0.55;
    drawEraPlaque(ctx, unitEra(u.kind as UnitKind), x, py, w, FONTS[look].display);
    drop = Math.max(drop, py + plaqueHeight(w) / 2 + 1.5 - (y + w * 1.04 + 3));
  }
  if (u.hp < 100) drawTokenHealth(ctx, look, u.hp, x, y, w, drop);
  if (u.military && u.level) drawPromotion(ctx, look, u.level, x, y, w, faction);
  const bx = x + w * 0.95;
  if (u.fortify > 0) {
    const ic = tintedIcon('fortify', TOKEN_INK.atlas.ink, 14 * dpr);
    if (ic) ctx.drawImage(ic, bx - 5, y + w * 0.35, 12, 12);
  }
  // The strength cue: an enemy your selected unit could attack this turn carries how much stronger or weaker it is in that fight.
  if (cue?.targets.has(u.id)) drawStrengthChip(ctx, matchupStrength(u, faction, cue.sel, false), matchupStrength(cue.sel, cue.selFaction, u, true), bx + 5, y - w * 0.85 - 5, FONTS[look].sans, 1, false, cue.neutralBarbarians && isBarbarian(u.owner));
  if (mark) drawBadge(ctx, look, bx, y - w * 0.85, w * 0.42, '', true, MARK_ICON[mark]);
  // An idle unit (short of a strategic resource) wears the alert mark until its supply returns; a pending order's badge takes the spot first.
  else if (mine && u.unsupplied) drawBadge(ctx, look, bx, y - w * 0.85, w * 0.42, '', true, 'ui-alert');
  else if (mine && u.standing_order) drawBadge(ctx, look, bx, y - w * 0.85, w * 0.38, '', false, u.standing_order === 'Heal' ? 'mark-heal' : u.standing_order === 'AutoFire' ? 'mark-autofire' : 'mark-sleep');
  else if (mine && u.goto) drawBadge(ctx, look, bx, y - w * 0.85, w * 0.38, '', false, 'mark-move');
  const tagY = y + w * 1.04 + (u.hp < 100 ? 10 : 4) + (mini ? w * 0.3 : 0) + Math.max(0, drop - (mini ? w * 0.8 : 0));
  // A hired Company wears its seller's seal and its turns of contract left, for every viewer (the countdown is public).
  if (contract !== null && contract !== undefined) drawContractTag(ctx, look, x, tagY, u.origin, contract);
  // Byzantium's Heir says whose he is and who holds him, so a rival knows whom to look for.
  else if (heirLabel) drawTag(ctx, look, x, tagY, heirLabel, mine ? 'good' : u.owner === u.heir_of ? 'neutral' : 'bad');
}

/** A warning roundel on the friend's tile that may stop the previewed route: "!" in attack ink. */
function drawRouteRisk(ctx: CanvasRenderingContext2D, S: MapStyle, p: { x: number; y: number }, hexPx: number) {
  const r = Math.max(7, hexPx * 0.17);
  const x = p.x + hexPx * 0.42, y = p.y - hexPx * 0.42;
  ctx.save();
  ctx.beginPath();
  ctx.arc(x, y, r, 0, Math.PI * 2);
  ctx.fillStyle = S.ink.markerFill;
  ctx.fill();
  ctx.strokeStyle = S.ink.attack;
  ctx.lineWidth = 1.6;
  ctx.stroke();
  ctx.fillStyle = S.ink.attack;
  ctx.font = `bold ${Math.round(r * 1.5)}px sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText('!', x, y + 0.5);
  ctx.restore();
}

function drawRoute(
  ctx: CanvasRenderingContext2D,
  inp: RenderInput,
  S: MapStyle,
  look: LookId,
  P: (h: Hex) => { x: number; y: number },
  unit: UnitView,
): Rect | null {
  const path = inp.path!;
  if (!path.length) return null;
  const color = inp.pathAttack ? S.ink.attack : S.ink.route;
  const pts = [P(unit.pos), ...path.map((st) => P(st.hex))];
  const last = pts[pts.length - 1];
  // Stop short of the final marker.
  const prev = pts[pts.length - 2];
  const d = Math.hypot(last.x - prev.x, last.y - prev.y) || 1;
  const cut = Math.min(12, d * 0.5);
  const end = { x: last.x - ((last.x - prev.x) / d) * cut, y: last.y - ((last.y - prev.y) / d) * cut };
  ctx.save();
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  const trace = () => {
    ctx.beginPath();
    ctx.moveTo(pts[0].x, pts[0].y);
    // Rounded through the hex centers: quadratic segments via midpoints.
    for (let k = 1; k < pts.length - 1; k++) {
      const m = k === pts.length - 2 ? end : { x: (pts[k].x + pts[k + 1].x) / 2, y: (pts[k].y + pts[k + 1].y) / 2 };
      ctx.quadraticCurveTo(pts[k].x, pts[k].y, m.x, m.y);
    }
    if (pts.length === 2) ctx.lineTo(end.x, end.y);
  };
  if (S.ink.routeShadow) {
    trace();
    ctx.strokeStyle = S.ink.routeShadow;
    ctx.lineWidth = 5;
    ctx.stroke();
  }
  trace();
  ctx.strokeStyle = color;
  ctx.lineWidth = 2;
  if (S.ink.routeDash) {
    ctx.setLineDash(S.ink.routeDash);
    ctx.lineDashOffset = -(inp.pulseTime ?? inp.time) / 60;
  }
  ctx.stroke();
  ctx.setLineDash([]);
  // Turn markers: roundels with the turn in roman numerals.
  for (const st of path) {
    if (!st.endsTurn) continue;
    const p = P(st.hex);
    const F = FONTS[look];
    ctx.lineWidth = 1.2;
    {
      ctx.beginPath();
      ctx.arc(p.x, p.y, 11, 0, Math.PI * 2);
      ctx.fillStyle = S.ink.markerFill;
      ctx.fill();
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.4;
      ctx.stroke();
      ctx.beginPath();
      ctx.arc(p.x, p.y, 8, 0, Math.PI * 2);
      ctx.lineWidth = 0.6;
      ctx.stroke();
      ctx.font = `700 13px ${F.display}`;
    }
    ctx.fillStyle = color;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(roman(st.turn), p.x, p.y + 1);
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
  }
  ctx.restore();
  if (!S.ink.routeDash) return null;
  // Everything the route draws: the line and its shadow, and the turn roundels (radius 11).
  const pad = 14;
  const xs = pts.map((p) => p.x), ys = pts.map((p) => p.y);
  const x0 = Math.min(...xs) - pad, y0 = Math.min(...ys) - pad;
  return { x: x0, y: y0, w: Math.max(...xs) + pad - x0, h: Math.max(...ys) + pad - y0 };
}
