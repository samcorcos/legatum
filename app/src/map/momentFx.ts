// Payoff moments on the map: a founded city takes its painted construction stages while its border spreads ring by ring,
// a queued wonder or Walls site steps forward with production, an ordinary building settles on its banner, a capture flips
// the banner and floods the land in the taker's colour, a town grows into its next size, and roads and improvements
// draw themselves in. legatum-ui: "a static state change at a payoff moment is a defect".
//
// Same layer as the combat animation (combatFx.ts, read its header): the plan is plain data built once per resolved
// turn from the viewer's own events and the views either side of it (nothing hidden is shown), and every frame is a
// pure function of the ms since the turn resolved, on the same speed-scaled clock as the combat playback. Drawing lives
// in momentFxDraw.ts.
//
// The moments follow the rules' resolution order (legatum-rules, "Resolution phases"): the moves and fights play
// first, then cities are founded, then production, growth and borders. Until its moment the map shows the state from
// before it: `holdAt` says what the terrain cache and the banners hold back (a town not yet founded, a tile not yet
// claimed, a banner not yet flipped), so the old state never flashes and the new one arrives with its animation.
import type { BuildingKind, Event, Hex, PlayerView, ProductionItem, Resource } from '../types';
import { RESOURCE_INFO, STRATEGIC_RESOURCES } from '../resources';
import { distance, hexEq, hexToPixel, tileIndex } from '../hex';
import { isGreatWork } from '../data';
import { HEX_SIZE } from './grid';
import { rnd, type CombatPlan, type Pt } from './combatFx';
import { sitesOf } from './constructionSites';

export interface FxFound { city: number; at: Hex; owner: number; tile: number; t: number; seed: number }
export interface FxBuild { city: number; at: Hex; tile: number; owner: number; item: ProductionItem; wonder: boolean; walls: boolean; from: number; to: number; t: number; seed: number }
/** A tile changing hands: claimed, founded around, captured or released. */
export interface FxTile { tile: number; hex: Hex; from: number | null; to: number | null; t: number; origin: Hex; seed: number }
export interface FxFlip { city: number; at: Hex; from: number; to: number; t: number }
/** A town reaching its next size tier, or rebuilt in its owner's new era: the art changes. */
export interface FxGrow { city: number; at: Hex; tile: number; fromPop: number; fromEra?: number; t: number; seed: number }
/** New road tiles drawn outward from `origin`, and the view's road bits before (held until the road reaches them). */
export interface FxRoad { origin: Hex; t: number; reach: number; tiles: Map<number, number | undefined> }
/** A seal stamped on a capital: a VP scored (gilt), an Alliance or Peace signed, a war declared. */
export type SealKind = 'vp' | 'alliance' | 'peace' | 'war';
export interface FxSeal { kind: SealKind; at: Hex; owner: number; vp: number; t: number; seed: number; /** Where it stands over its city: 0 above the banner, then left and right of it, so nothing overlaps. */ slot: number }
/** A unit promoted or upgraded: a glint across its counter. */
export interface FxGlint { unit: number; t: number; seed: number }
export interface FxImprove { tile: number; hex: Hex; before: PlayerView['tiles'][number]['improvement']; t: number; seed: number }
/** A newly known deposit, held off the map until its ink medallion has been drawn. */
export interface FxResourceReveal { tile: number; hex: Hex; resource: Resource; t: number }

/** An original capital taken this turn: its star stays filled until the VP seal has landed, then goes hollow (scored). */
export interface FxCapital { city: number; t: number }

export interface MomentPlan {
  total: number;
  founds: FxFound[];
  builds: FxBuild[];
  tiles: FxTile[];
  flips: FxFlip[];
  grows: FxGrow[];
  roads: FxRoad[];
  improves: FxImprove[];
  reveals: FxResourceReveal[];
  seals: FxSeal[];
  glints: FxGlint[];
  capitals: FxCapital[];
}

/** Painted construction sites and their turn steps are active. MapCanvas still gates every moment
 * behind Map animations On and reduced motion; Off shows the resulting map state immediately. */
export const CONSTRUCTION_ANIMATION = false;

/** Timings (ms from the moment's own start). */
export const FOUND = { ink: 120, inkEnd: 1050, wash: 560, washEnd: 1500, done: 1560, ring0: 260, ringStep: 420, banner: 1150, bannerEnd: 1520 };
/** An ordinary banner beat, a site step, and the final construction step. */
export const BUILD_MS = { step: 1200, finish: 2500, building: 1000 };
export const FLIP_MS = 560;
export const TILE_MS = 700;
export const GROW_MS = 1100;
export const ROAD_MS = 1300;
export const IMPROVE_MS = 900;
export const SEAL_MS = 1900;
/** How long into its VP seal a captured capital's star holds before going hollow. */
export const CAPITAL_STAR_LAND = 650;
export const GLINT_MS = 900;
export const RESOURCE_REVEAL_MS = 850;
/** Moments of one kind fire this far apart, so a turn with many reads as a sequence, capped so the last is not late. */
const STAGGER = 380;
const STAGGER_CAP = 2600;

export const worldOf = (h: Hex): Pt => hexToPixel(h, HEX_SIZE);
const TAIL = 200;
/**
 * How long the overlay keeps drawing a finished town or walls after the cache takes them back: the cache repaints the
 * chunk within its frame budget, which can take a frame or two, and the town must never blink out meanwhile.
 */
export const LINGER = 250;

/** A build's phases (ms from its start): done when the scaffold is down, the flourish just before, settled once it has joined its town. */
export function buildPhases(b: Pick<FxBuild, 'wonder' | 'walls' | 'to'>) {
  const finished = b.to >= 1;
  const done = !b.wonder && !b.walls ? BUILD_MS.building : finished ? BUILD_MS.finish : BUILD_MS.step;
  return { done, flourish: Math.round(done * .9), settled: done };
}

/** When the moves and fights of the turn are over: the combat plan's last step, else the plain glide. */
export function movesEnd(plan: CombatPlan | null | undefined, glideMs: number): number {
  if (!plan) return glideMs;
  let end = plan.volley;
  for (const steps of plan.moves.values()) for (const s of steps) end = Math.max(end, s.t1);
  return end;
}

/**
 * The turn's payoff moments, from the viewer's events and views; null when there are none. `start` is when the moves
 * end (`movesEnd`); a capture lands when the combat plan takes the city (`captureAt`), else at `start`.
 */
export function buildMomentPlan(events: readonly Event[], before: PlayerView | null, after: PlayerView, start: number, captureAt?: (hex: Hex) => number | null): MomentPlan | null {
  if (!before) return null;
  const W = after.width, H = after.height;
  const cityOf = (id: number) => after.cities.find((c) => c.id === id) ?? before.cities.find((c) => c.id === id) ?? null;
  const visible = (h: Hex) => { const i = tileIndex(h, W, H); return i >= 0 && !!after.tiles[i]?.visible; };
  const foundAt = start + 150;
  const econ = start + 500;

  const founds: FxFound[] = [];
  const flips: FxFlip[] = [];
  const builds: FxBuild[] = [];
  const beforeSites = new Map(sitesOf(before).map(s => [s.city, s]));
  const afterSites = new Map(sitesOf(after).map(s => [s.city, s]));
  const grows: FxGrow[] = [];
  events.forEach((e, i) => {
    if (e.type === 'CityFounded' && visible(e.at)) {
      founds.push({ city: e.city, at: e.at, owner: e.owner, tile: tileIndex(e.at, W, H), t: foundAt + Math.min(founds.length * STAGGER, STAGGER_CAP), seed: 11 + i * 7 });
    } else if (e.type === 'CityCaptured') {
      const c = cityOf(e.city);
      if (!c || !visible(c.pos)) return;
      flips.push({ city: e.city, at: c.pos, from: e.from, to: e.to, t: captureAt?.(c.pos) ?? start });
    } else if (e.type === 'ProductionCompleted' && (e.item.kind === 'Building' || e.item.kind === 'Wonder')) {
      const c = cityOf(e.city);
      const at = e.item.kind === 'Wonder' ? e.item.tile : c?.pos;
      if (!c || !at || !visible(at)) return;
      const b: BuildingKind = e.item.building;
      const previous = beforeSites.get(e.city);
      builds.push({ city: e.city, at, tile: tileIndex(at, W, H), owner: e.owner, item: e.item,
        wonder: e.item.kind === 'Wonder' || isGreatWork(b), walls: b === 'Walls', from: previous?.k ?? 0, to: 1, t: 0, seed: 23 + i * 13 });
    } else if (e.type === 'CityGrew') {
      const old = before.cities.find((c) => c.id === e.city);
      const c = cityOf(e.city);
      if (!old || !c || !visible(c.pos) || tierOf(old.population) === tierOf(e.population)) return;
      grows.push({ city: e.city, at: c.pos, tile: tileIndex(c.pos, W, H), fromPop: old.population, t: 0, seed: 31 + i * 5 });
    }
  });
  // A site is a map state: its step occurs on every turn with more production, even without an event.
  for (const site of afterSites.values()) {
    if (site.owner !== after.me || !visible(site.hex) || builds.some(b => b.city === site.city)) continue;
    const previous = beforeSites.get(site.city);
    if (!before.cities.some((c) => c.id === site.city) || (previous && (previous.building !== site.building || previous.tile !== site.tile))) continue;
    const from = previous?.k ?? 0;
    if (site.k <= from) continue;
    const item: ProductionItem = site.walls ? { kind: 'Building', building: 'Walls' } : { kind: 'Wonder', building: site.building, tile: site.hex };
    builds.push({ city: site.city, at: site.hex, tile: site.tile, owner: site.owner, item,
      wonder: !site.walls, walls: site.walls, from, to: site.k, t: 0, seed: 23 + site.seed });
  }
  // An empire entering an era rebuilds its towns in the new style: a wave out from its capital.
  for (const e of events) {
    if (e.type !== 'EraEntered' || e.era < 2) continue;
    const mine = after.cities.filter((c) => c.owner === e.player && c.visible && visible(c.pos));
    const capital = mine.find((c) => c.is_capital) ?? mine[0];
    for (const c of mine) {
      if (grows.some((g) => g.city === c.id)) continue;
      grows.push({ city: c.id, at: c.pos, tile: tileIndex(c.pos, W, H), fromPop: c.population, fromEra: e.era - 2,
        t: econ + 400 + Math.min(distance(capital.pos, c.pos) * 110, STAGGER_CAP), seed: 61 + c.id * 3 });
    }
  }
  // Production in the order the cities finished it; wonders last, so the biggest moment closes the turn.
  builds.sort((a, b) => Number(a.wonder) - Number(b.wonder));
  builds.forEach((b, k) => { b.t = econ + Math.min(k * STAGGER, STAGGER_CAP); });
  grows.forEach((g, k) => { if (g.fromEra === undefined) g.t = econ + 200 + Math.min(k * STAGGER, STAGGER_CAP); });

  // Borders: every explored tile whose owner changed, timed from the city it now (or last) belongs to.
  const tiles: FxTile[] = [];
  const claims = new Map<string, number>();
  for (const e of events) if (e.type === 'TileClaimed' || e.type === 'TileBought' || e.type === 'TileReleased') claims.set(`${e.tile.q},${e.tile.r}`, e.city);
  let claimN = 0;
  for (let i = 0; i < after.tiles.length; i++) {
    const a = after.tiles[i], b = before.tiles[i];
    if (!a?.explored || !b?.explored || a.owner === b.owner) continue;
    const hex = hexOf(i, W);
    const found = founds.find((f) => a.owner === f.owner && distance(f.at, hex) <= 3);
    const flip = flips.find((f) => (a.owner === f.to && b.owner === f.from) && distance(f.at, hex) <= 4);
    let t: number, origin: Hex;
    if (found) {
      origin = found.at;
      const d = distance(found.at, hex);
      t = found.t + FOUND.ring0 + d * FOUND.ringStep + angleOf(found.at, hex) * 0.18 * FOUND.ringStep;
    } else if (flip) {
      origin = flip.at;
      t = flip.t + FLIP_MS * 0.5 + distance(flip.at, hex) * 240 + angleOf(flip.at, hex) * 60;
    } else {
      const city = claims.get(`${hex.q},${hex.r}`);
      const c = city != null ? cityOf(city) : nearestCity(after, hex, a.owner ?? b.owner);
      origin = c?.pos ?? hex;
      t = econ + 300 + Math.min(claimN++ * 220, STAGGER_CAP);
    }
    tiles.push({ tile: i, hex, from: b.owner, to: a.owner, t, origin, seed: 41 + i });
  }

  // Roads: new road tiles, drawn outward from the city that laid them (else the nearest own city).
  const roads: FxRoad[] = [];
  const laid = events.filter((e): e is Extract<Event, { type: 'RoadsLaid' }> => e.type === 'RoadsLaid');
  const newRoad = new Map<number, number | undefined>();
  for (let i = 0; i < after.tiles.length; i++) {
    const a = after.tiles[i], b = before.tiles[i];
    if (a?.explored && a.visible && (a.road ?? 0) !== (b?.road ?? 0) && a.road) newRoad.set(i, b?.road);
  }
  if (newRoad.size) {
    const origins = laid.length ? laid.map((e) => e.at) : [nearestCity(after, hexOf(newRoad.keys().next().value!, W), after.me)?.pos ?? hexOf(newRoad.keys().next().value!, W)];
    origins.forEach((o, k) => {
      const mine = new Map([...newRoad].filter(([i]) => nearestOf(origins, hexOf(i, W)) === k));
      if (!mine.size) return;
      const reach = Math.max(1, ...[...mine.keys()].map((i) => distance(o, hexOf(i, W))));
      roads.push({ origin: o, t: econ + 400 + k * STAGGER, reach, tiles: mine });
    });
  }

  // Improvements: a farm's furrows or a mine cut in, tile by tile.
  const improves: FxImprove[] = [];
  for (let i = 0; i < after.tiles.length; i++) {
    const a = after.tiles[i], b = before.tiles[i];
    if (!a?.visible || !a.improvement) continue;
    if (b?.improvement?.kind === a.improvement.kind && !!b.improvement.upgraded === !!a.improvement.upgraded) continue;
    improves.push({ tile: i, hex: hexOf(i, W), before: b?.improvement ?? null, t: econ + 150 + Math.min(improves.length * 260, STAGGER_CAP), seed: 53 + i });
  }

  const learned = new Set(events.filter((e): e is Extract<Event, { type: 'TechResearched' }> => e.type === 'TechResearched' && e.player === after.me).map((e) => e.tech));
  const revealed = STRATEGIC_RESOURCES.filter((r) => RESOURCE_INFO[r].revealedBy && learned.has(RESOURCE_INFO[r].revealedBy!));
  const reveals: FxResourceReveal[] = [];
  if (revealed.length) after.tiles.forEach((tile, index) => {
    if (!tile.explored || !tile.resource || !revealed.includes(tile.resource) || before.tiles[index]?.resource === tile.resource) return;
    reveals.push({ tile: index, hex: hexOf(index, W), resource: tile.resource,
      t: econ + 120 + Math.min(reveals.length * 110, STAGGER_CAP) });
  });

  // Seals: diplomacy is settled first in the turn, the score last (legatum-rules, resolution phases).
  const seals: FxSeal[] = [];
  const capitals: FxCapital[] = [];
  const capitalOf = (player: number) => after.cities.find((c) => c.owner === player && c.is_capital && visible(c.pos)) ?? null;
  const teamCapitals = (team: number) => after.players.filter((p) => p.team === team).map((p) => capitalOf(p.id)).filter((c): c is NonNullable<typeof c> => !!c);
  const scoreAt = econ + 900;
  events.forEach((e, i) => {
    const seal = (kind: SealKind, player: number, vp: number, t: number) => {
      const c = capitalOf(player);
      if (!c) return;
      // One seal per empire for its VP this turn: five objectives read as "+5 VP", not five stamps on one city.
      const same = kind === 'vp' ? seals.find((x) => x.kind === 'vp' && x.owner === player) : undefined;
      if (same) { same.vp += vp; return; }
      seals.push({ kind, at: c.pos, owner: player, vp, t: t + Math.min(seals.filter((x) => x.kind === kind).length * STAGGER, STAGGER_CAP), seed: 71 + i * 3, slot: 0 });
    };
    if (e.type === 'EraObjectiveScored') seal('vp', e.player, 1, scoreAt);
    else if (e.type === 'FactionGoalCompleted') seal('vp', e.player, e.vp, scoreAt);
    else if (e.type === 'GreatWorkCompleted') seal('vp', e.player, e.vp, scoreAt);
    else if (e.type === 'CapitalTaken') { seal('vp', e.captor, 1, scoreAt); capitals.push({ city: e.capital, t: (seals.find((x) => x.kind === 'vp' && x.owner === e.captor)?.t ?? scoreAt) + CAPITAL_STAR_LAND }); }
    else if (e.type === 'TreatyActivated' || e.type === 'WarDeclared' || e.type === 'PeaceEstablished') {
      const kind: SealKind = e.type === 'WarDeclared' ? 'war' : e.relation.state === 'Alliance' ? 'alliance' : 'peace';
      for (const team of [e.relation.a, e.relation.b]) for (const c of teamCapitals(team)) {
        if (!seals.some((x) => x.kind === kind && hexEq(x.at, c.pos))) seals.push({ kind, at: c.pos, owner: c.owner, vp: 0, t: 250 + seals.length * 160, seed: 81 + i * 5 + c.id, slot: 0 });
      }
    }
  });

  // Slots: a seal never covers a rising building or another seal on the same city while both are up.
  for (const x of seals) {
    const busy = new Set<number>();
    for (const b of CONSTRUCTION_ANIMATION ? builds : []) if (hexEq(b.at, x.at) && b.t < x.t + SEAL_MS && x.t < b.t + buildPhases(b).settled) busy.add(0);
    for (const y of seals) if (y !== x && hexEq(y.at, x.at) && y.slot >= 0 && seals.indexOf(y) < seals.indexOf(x) && y.t < x.t + SEAL_MS && x.t < y.t + SEAL_MS) busy.add(y.slot);
    let k = 0;
    while (busy.has(k)) k++;
    x.slot = k;
  }

  // Promotions and upgrades (empire actions and the healing phase): a glint on each counter, after the moves.
  const glints: FxGlint[] = [];
  events.forEach((e, i) => {
    if (e.type !== 'UnitPromoted' && e.type !== 'UnitUpgraded') return;
    const u = after.units.find((x) => x.id === e.unit);
    if (u && visible(u.pos)) glints.push({ unit: e.unit, t: start + 250 + Math.min(glints.length * 140, STAGGER_CAP), seed: 91 + i });
  });

  if (!glints.length && !seals.length && !founds.length && !builds.length && !tiles.length && !flips.length && !grows.length && !roads.length && !improves.length && !reveals.length) return null;
  const ends = [
    ...(CONSTRUCTION_ANIMATION ? founds.map((f) => f.t + FOUND.done + LINGER) : []),
    ...(CONSTRUCTION_ANIMATION ? builds.map((b) => b.t + buildPhases(b).settled + LINGER + 600) : []),
    ...tiles.map((x) => x.t + TILE_MS),
    ...flips.map((f) => f.t + FLIP_MS),
    ...(CONSTRUCTION_ANIMATION ? grows.map((g) => g.t + GROW_MS + LINGER) : []),
    ...roads.map((r) => r.t + ROAD_MS),
    ...(CONSTRUCTION_ANIMATION ? improves.map((m) => m.t + IMPROVE_MS) : []),
    ...seals.map((x) => x.t + SEAL_MS),
    ...glints.map((x) => x.t + GLINT_MS),
    ...reveals.map((x) => x.t + RESOURCE_REVEAL_MS),
  ];
  return { total: Math.max(0, ...ends) + TAIL, founds, builds, tiles, flips, grows, roads, improves, reveals, seals, glints, capitals };
}

/** Is any moment still playing at `t`? */
export const momentsLive = (plan: MomentPlan, t: number) => t >= 0 && t < plan.total;

/** What the map holds back at `t`: everything still waiting for (or in the middle of) its animation. */
export interface MomentHold {
  /** Freshly known deposit icons withheld while their ink appears. */
  resources: Set<number>;
  /** Tiles whose town the terrain cache must not draw (the overlay draws it). */
  towns: Set<number>;
  /** Tiles whose walls the terrain cache must not draw. */
  walls: Set<number>;
  /** Site progress held at the prior view until its step plays. */
  sites: Map<number, number>;
  /** Tiles whose town the cache draws at another population (the size tier before it grew). */
  pop: Map<number, number>;
  /** Tiles whose town the cache draws in an earlier era (before its owner's era-up reached it). */
  era: Map<number, number>;
  /** Tile owners as the cache should paint them. */
  owner: Map<number, number | null>;
  /** City owners as their banners show them (before the flip). */
  cityOwner: Map<number, number>;
  /** Original capitals just taken: their star stays filled (not yet scored) until the VP seal lands. */
  unscored: Set<number>;
  /** Banner motion: alpha, a vertical drop (CSS px) and a horizontal scale (the flip). */
  banner: Map<number, { alpha: number; dy: number; sx: number }>;
  /** Changes whenever anything above changes that the terrain cache paints (towns, walls, pop, owners). */
  sig: string;
}

const clamp01 = (k: number) => Math.max(0, Math.min(1, k));
const easeOut = (k: number) => 1 - (1 - k) * (1 - k);

export function holdAt(plan: MomentPlan, t: number): MomentHold {
  const resources = new Set(plan.reveals.filter((x) => t < x.t + RESOURCE_REVEAL_MS).map((x) => x.tile));
  const towns = new Set<number>(), walls = new Set<number>(), sites = new Map<number, number>(), pop = new Map<number, number>(), era = new Map<number, number>();
  const owner = new Map<number, number | null>(), cityOwner = new Map<number, number>();
  const banner = new Map<number, { alpha: number; dy: number; sx: number }>();
  const unscored = new Set<number>(plan.capitals.filter((x) => t < x.t).map((x) => x.city));
  for (const f of CONSTRUCTION_ANIMATION ? plan.founds : []) {
    const dt = t - f.t;
    if (dt < FOUND.done) towns.add(f.tile);
    if (dt < FOUND.bannerEnd) {
      const k = clamp01((dt - FOUND.banner) / (FOUND.bannerEnd - FOUND.banner));
      banner.set(f.city, { alpha: easeOut(k), dy: (1 - easeOut(k)) * -6, sx: 1 });
    }
  }
  for (const b of CONSTRUCTION_ANIMATION ? plan.builds : []) {
    if (t < b.t + buildPhases(b).done) {
      sites.set(b.city, b.from);
      if (b.walls && b.to >= 1) walls.add(b.tile);
    }
  }
  for (const g of CONSTRUCTION_ANIMATION ? plan.grows : []) if (t < g.t + GROW_MS) { pop.set(g.tile, g.fromPop); if (g.fromEra !== undefined) era.set(g.tile, g.fromEra); }
  for (const x of plan.tiles) if (t < x.t) owner.set(x.tile, x.from);
  for (const f of plan.flips) {
    const k = (t - f.t) / FLIP_MS;
    if (k < 0.5) cityOwner.set(f.city, f.from);
    if (k >= 0 && k < 1) banner.set(f.city, { alpha: 1, dy: -Math.sin(Math.PI * k) * 6, sx: Math.max(0.04, Math.abs(Math.cos(Math.PI * k))) });
  }
  const sig = [...towns].join(',') + '|' + [...walls].join(',') + '|' + [...pop].join(',') + '|' + [...era].join(',') + '|' + [...owner.keys()].join(',');
  return { resources, towns, walls, sites, pop, era, owner, cityOwner, unscored, banner, sig };
}

/** When the plan's moment at `hex` starts (a founding, a finished building), so the map's own celebration waits for it. */
export function momentTime(plan: MomentPlan, hex: Hex): number | null {
  if (!CONSTRUCTION_ANIMATION) return null;
  const f = plan.founds.find((x) => hexEq(x.at, hex));
  if (f) return f.t + FOUND.wash;
  const b = plan.builds.find((x) => hexEq(x.at, hex));
  if (b) return b.t + buildPhases(b).flourish;
  return null;
}

// ---- marching ----

/** A unit on the move: a lift (world px, negative is up) for its stride, and where its heels were a moment ago. */
export interface March { lift: number; trail: Pt[] }
/** One stride (ms): the cadence of the march. The lift scales with the speed, so a unit easing in or out steps lightly. */
const STRIDE_MS = 240;
/** A marching unit at `t` (ms), from where it stands over time (`at`): null while it stands still. */
export function marchOf(at: (t: number) => Pt, t: number, moving: boolean): March | null {
  if (!moving) return null;
  const p = at(t);
  const trail = [at(t - 70), at(t - 140), at(t - 210)];
  const speed = Math.hypot(p.x - trail[0].x, p.y - trail[0].y);
  if (speed < 0.3) return null;
  return { lift: -Math.abs(Math.sin((t / STRIDE_MS) * Math.PI)) * 2.4 * Math.min(1, speed / 3), trail };
}

// ---- helpers ----

function tierOf(pop: number) { return pop >= 13 ? 3 : pop >= 8 ? 2 : pop >= 4 ? 1 : 0; }

function hexOf(i: number, width: number): Hex {
  const col = i % width, row = Math.floor(i / width);
  return { q: col - (row - (row & 1)) / 2, r: row };
}

/** 0..1 clockwise from north around `c`, so a ring is swept rather than all at once. */
function angleOf(c: Hex, h: Hex): number {
  const a = worldOf(c), b = worldOf(h);
  return ((Math.atan2(b.x - a.x, -(b.y - a.y)) / (Math.PI * 2)) + 1) % 1;
}

function nearestCity(view: PlayerView, h: Hex, owner: number | null) {
  let best: PlayerView['cities'][number] | null = null, bd = Infinity;
  for (const c of view.cities) {
    if (owner != null && c.owner !== owner) continue;
    const d = distance(c.pos, h);
    if (d < bd) { bd = d; best = c; }
  }
  return best;
}

function nearestOf(list: Hex[], h: Hex): number {
  let k = 0, bd = Infinity;
  list.forEach((o, i) => { const d = distance(o, h); if (d < bd) { bd = d; k = i; } });
  return k;
}

/** Deterministic noise, shared with the combat layer. */
export { rnd };
