// Combat animation: the turn's fighting played back on the map in the order the rules resolved it (legatum-rules,
// resolution phases): first every shot of the ranged phase at once (arrows, bolts, shot and shells, city walls
// answering), then the movement rounds with each melee clash on the step where it happened, losses and captures.
//
// The plan is built once per resolved turn from the viewer's own events (nothing hidden is shown) and every frame is a
// pure function of the time since the turn resolved, so playback is deterministic, seekable (the media captures step it
// 1/60 s at a time) and testable without a canvas. Drawing lives in combatFxDraw.ts.
//
// The animation layer, for anyone adding a map effect (combat here; other payoff moments can follow the same shape):
//   - No library: effects are pure functions of elapsed ms, drawn by the map's own rAF loop on the existing canvas
//     (MapCanvas.tsx draw loop -> render.ts `inp.fx` -> combatFxDraw.ts). That keeps them deterministic, so the media
//     captures can frame-step them on Playwright's clock, and cheap (strokes, fills and pre-rendered wash sprites; no
//     shadowBlur, filters or per-frame canvases). Only canvas 2D calls WebKit (Safari) supports.
//   - Plan once, draw many: build a plain-data plan when the turn resolves (store.ts applyTurn attaches it to `anim`),
//     then each frame asks what is on screen at t (`fxUnits` for unit offsets, `drawCombatFx` for the effects).
//   - Gate every effect on `useCombatFx()` (feel/combatFxPreferences.ts: the Settings toggle and speed) and on
//     `prefers-reduced-motion` (MapCanvas `reducedMotion`): off means the plain behaviour, never a half effect.
//   - Keep the loop alive only while something plays (`fxLive`), so an idle map draws nothing extra.
//   - Engraved building blocks in combatFxDraw.ts: `cloud` (engraved puffs: dust, smoke, soot), `rays` (woodcut
//     sunburst), `flash`, `ring`, `debris`. Reuse them so every effect shares one line language (legatum-art).
import type { Event, Hex, PlayerView, UnitKind, UnitView } from '../types';
import { attackerUnit } from '../types';
import { hexToPixel, hexEq } from '../hex';
import { unitClass, unitEraTier } from '../data';
import { siegePropFor } from '../siegeProps';
import { HEX_SIZE } from './grid';

export interface Pt { x: number; y: number }

/** The aircraft kinds a strike can fly. */
const AIR_CRAFT = ['Biplane', 'Bomber', 'JetBomber', 'Fighter', 'JetFighter', 'StealthBomber', 'StealthFighter'] as const;
/** What flies: by the shooter's kind and era (a city's walls by its owner's era). */
export type ShotKind = 'arrows' | 'bolts' | 'shot' | 'stone' | 'ball' | 'shell' | 'drone' | 'bombs';

export interface FxShot {
  kind: ShotKind;
  /** World px: the shooter (a city's wall facing the target) and the target. */
  from: Pt;
  to: Pt;
  launch: number;
  flight: number;
  /** It did damage (a red spark on impact); a miss still lands. */
  hit: boolean;
  /** Fired by a city's walls. */
  city: boolean;
  /** Aimed at a city (stones and dust off the walls). */
  atCity: boolean;
  /** The defending unit, which flinches on impact. */
  target: number | null;
  /** The firing unit, which recoils as it fires (null for a city). */
  shooter: number | null;
  /** An air strike's aircraft kind, for its painted plan sprite. */
  craft?: (typeof AIR_CRAFT)[number];
  seed: number;
}

export interface FxClash {
  attacker: number | null;
  defender: number | null;
  from: Pt;
  to: Pt;
  /** The moment of contact. */
  t: number;
  /** The attacker lunges and falls back (it did not step in this round); otherwise its own step is the charge. */
  lunge: boolean;
  mounted: boolean;
  /** Gunpowder era or later: muskets and rifles fire as the lines close. */
  guns: boolean;
  atCity: boolean;
  hit: boolean;
  seed: number;
}

export interface FxDeath { unit: UnitView; t: number; seed: number }
export interface FxCapture { city: number; at: Pt; t: number; owner: number; razed: boolean; orderedRaze: boolean }
/**
 * One continuous glide along a path: a unit's steps in consecutive rounds run together as one smooth move (eased only
 * at its ends), so a 3-hex charge is one charge, not three hops. Its rounds keep their timing (the middle runs at an
 * even pace, one hex per round), so units still meet where and when the rules had them meet. A unit that waits a
 * round starts a new glide.
 */
export interface FxStep { t0: number; t1: number; path: Pt[]; from: Pt; to: Pt }

export interface CombatPlan {
  /** When the movement rounds start (the ranged phase is over). */
  volley: number;
  stepMs: number;
  total: number;
  shots: FxShot[];
  clashes: FxClash[];
  deaths: FxDeath[];
  captures: FxCapture[];
  /** Every moving unit's steps, round by round (a unit destroyed this turn too). */
  moves: Map<number, FxStep[]>;
  /** Units destroyed this turn, drawn from the view before the turn until they fall. */
  ghosts: Map<number, UnitView>;
  /** Per hex ("q,r"): when its first blow lands, so its damage cue appears then. */
  cueDelay: Map<string, number>;
  /** HP over the playback, per unit ("u7") and city ("c3"): the bars drop as each blow lands. */
  hp: Map<string, { t: number; hp: number }[]>;
}

export const FLIGHT_MS: Record<ShotKind, number> = { arrows: 680, bolts: 440, shot: 170, stone: 920, ball: 540, shell: 700, drone: 560, bombs: 760 };
/** Peak height of the arc as a share of the distance flown. */
export const ARC: Record<ShotKind, number> = { arrows: 0.34, bolts: 0.12, shot: 0.02, stone: 0.48, ball: 0.17, shell: 0.4, drone: 0.06, bombs: 0 };
// Short volleys stay 150 to 300 ms apart (a volley and its answer read even in a 1.4 s trailer shot).
const STAGGER_MS = 190;
const STAGGER_CAP_MS = 1100;
const VOLLEY_TAIL_MS = 220;
const STEP_MS = 280;
const MOVE_BUDGET_MS = 1400;
/** How long a fallen unit takes to go, and the longest after-effect (a capture). */
export const DEATH_MS = 700;
export const CAPTURE_MS = 1300;
const TAIL_MS = 500;

const key = (h: Hex) => `${h.q},${h.r}`;
export const world = (h: Hex): Pt => hexToPixel(h, HEX_SIZE);

/** Deterministic noise in [0, 1) for (seed, i). */
export function rnd(seed: number, i: number): number {
  let x = (seed * 374761393 + i * 668265263) | 0;
  x = Math.imul(x ^ (x >>> 13), 1274126177);
  x ^= x >>> 16;
  return (x >>> 0) / 4294967296;
}

/** The projectile a unit fires (null: it does not shoot). A siege engine's crew throws stones or shot at a city. */
export function shotKindFor(kind: UnitKind | null, era: number, siege: string | null, atCity: boolean): ShotKind {
  if (atCity && siege) return siege === 'bombard' ? 'ball' : 'stone';
  switch (kind) {
    case 'Ranger': case 'CompositeArcher': case 'Cataphract': return 'arrows';
    case 'Marksman': return 'bolts';
    case 'Arquebusier': return 'shot';
    case 'FieldCannon': return 'ball';
    case 'Howitzer': case 'Artillery': case 'RocketArtillery': return 'shell';
    case 'Drone': return 'drone';
    case 'Biplane': case 'Bomber': case 'JetBomber': case 'Fighter': case 'JetFighter': return 'bombs';
    default: return cityShotKind(era);
  }
}

/** A city's walls fire by its owner's era (0-based): arrows, then bolts, cannon and shells. */
export function cityShotKind(era: number): ShotKind {
  return era <= 1 ? 'arrows' : era === 2 ? 'bolts' : era === 3 ? 'ball' : 'shell';
}

/**
 * The turn's combat as a plan, from the viewer's events; null when nothing was fought (plain moves keep the glide).
 * `before` is the viewer's view from before the turn (for shooters and units that died), `after` the new one.
 */
export function buildCombatPlan(events: Event[], before: PlayerView | null, after: PlayerView): CombatPlan | null {
  if (!events.some((e) => e.type === 'Battle' || e.type === 'UnitDestroyed' || e.type === 'CityCaptured' || e.type === 'CityRazed')) return null;
  const unitOf = (id: number | null): UnitView | null => id == null ? null
    : after.units.find((u) => u.id === id) ?? before?.units.find((u) => u.id === id) ?? null;
  const cityOf = (id: number) => after.cities.find((c) => c.id === id) ?? before?.cities.find((c) => c.id === id) ?? null;
  const eraOf = (player: number) => Math.max(0, (after.players.find((p) => p.id === player)?.era ?? 1) - 1);
  const craftOf = new Map([...(before?.aircraft ?? []), ...(after.aircraft ?? [])].map((a) => [a.id, a.kind]));
  const aircraft = new Set(craftOf.keys());

  // Movement rounds: every unit takes at most one step per round, so a unit stepping again opens the next round.
  type Step = { unit: number; from: Hex; to: Hex; round: number };
  const steps: Step[] = [];
  const battleRound = new Map<number, number>();
  let round = 0;
  let moved = new Set<number>();
  events.forEach((e, i) => {
    if (e.type === 'UnitMoved' && e.from && e.to) {
      if (moved.has(e.unit)) { round++; moved = new Set(); }
      moved.add(e.unit);
      steps.push({ unit: e.unit, from: e.from, to: e.to, round });
    } else if (e.type === 'Battle' && !e.ranged) battleRound.set(i, round);
  });
  const rounds = steps.length ? Math.max(...steps.map((s) => s.round)) + 1 : 0;

  // The ranged phase: every shot in the order it resolved, staggered so a volley reads as a volley.
  const shots: FxShot[] = [];
  const shotCount = events.filter((e) => e.type === 'Battle' && e.ranged).length;
  // At very high counts the 60 ms floor takes precedence over the 1.1 s target span;
  // clamping launch times to that span would make the final shots fire together.
  const shotGap = shotCount <= 6 ? STAGGER_MS : Math.max(60, Math.min(STAGGER_MS, STAGGER_CAP_MS / (shotCount - 1)));
  const impact = new Map<number, number>();
  const cueDelay = new Map<string, number>();
  const firstAt = (h: Hex, t: number) => { const k = key(h); if (!cueDelay.has(k) || cueDelay.get(k)! > t) cueDelay.set(k, t); };
  events.forEach((e, i) => {
    if (e.type !== 'Battle' || !e.ranged) return;
    const id = attackerUnit(e.attacker);
    const city = typeof e.attacker === 'object' ? cityOf(e.attacker.city) : null;
    const shooter = unitOf(id);
    const fromHex = e.attacker_at ?? shooter?.pos ?? city?.pos ?? null;
    const atCity = e.defender.kind === 'City' || after.cities.some((c) => hexEq(c.pos, e.at));
    let kind: ShotKind;
    if (city) kind = cityShotKind(eraOf(city.owner));
    else if (id != null && aircraft.has(id)) kind = 'bombs';
    else if (shooter) kind = shotKindFor(shooter.kind as UnitKind, unitEraTier(shooter.kind as UnitKind), siegePropFor(before ?? after, shooter), atCity);
    else kind = cityShotKind(eraOf(e.attacker_owner));
    const n = shots.length;
    const seed = 1 + i * 31 + (id ?? 0) * 7;
    const flight = FLIGHT_MS[kind] * (1 + rnd(seed, 2) * 0.08);
    // An air strike's aircraft crosses the map for half its pass (1.3 flights) before it is over the target and releases.
    const launch = n * shotGap + (shotCount <= 6 ? rnd(seed, 1) * 40 : 0) + (kind === 'bombs' ? flight * 1.3 : 0);
    const to = world(e.at);
    // An unseen shooter's fire still lands, from a little way off toward where it came from (or straight down).
    let from = fromHex ? world(fromHex) : { x: to.x - HEX_SIZE * 1.4, y: to.y - HEX_SIZE * 0.6 };
    if (city && fromHex) {
      // A city fires from its wall on the side facing its target.
      const d = Math.hypot(to.x - from.x, to.y - from.y) || 1;
      from = { x: from.x + (to.x - from.x) / d * HEX_SIZE * 0.55, y: from.y + (to.y - from.y) / d * HEX_SIZE * 0.55 };
    }
    shots.push({ kind, from, to, launch, flight, hit: e.damage_to_defender > 0, city: !!city, atCity, target: e.defender.kind === 'Unit' ? e.defender.unit : null, shooter: city ? null : id, seed, ...(kind === 'bombs' ? { craft: AIR_CRAFT.find((k) => k === craftOf.get(id ?? -1)) ?? 'Bomber' as const } : {}) });
    impact.set(i, launch + flight);
    firstAt(e.at, launch + flight);
  });
  const volley = shots.length ? Math.max(...shots.map((s) => s.launch + s.flight)) + VOLLEY_TAIL_MS : 0;
  const stepMs = rounds ? Math.min(STEP_MS, MOVE_BUDGET_MS / rounds) : STEP_MS;

  const moves = new Map<number, FxStep[]>();
  for (const s of steps) {
    const t0 = volley + s.round * stepMs;
    const list = moves.get(s.unit) ?? [];
    const last = list[list.length - 1];
    const to = world(s.to);
    // The next round's step continues the glide; a round spent waiting ends it.
    if (last && Math.abs(last.t1 - (t0 - stepMs * 0.06)) < 1 && last.to.x === world(s.from).x && last.to.y === world(s.from).y) {
      last.path.push(to);
      last.to = to;
      last.t1 = t0 + stepMs * 0.94;
    } else list.push({ t0, t1: t0 + stepMs * 0.94, path: [world(s.from), to], from: world(s.from), to });
    moves.set(s.unit, list);
  }

  // Melee: the clash lands mid-step of its round.
  const clashes: FxClash[] = [];
  events.forEach((e, i) => {
    if (e.type !== 'Battle' || e.ranged) return;
    const r = battleRound.get(i) ?? 0;
    const id = attackerUnit(e.attacker);
    const a = unitOf(id);
    const t = volley + r * stepMs + stepMs * 0.5;
    const steppedIn = id != null && steps.some((s) => s.unit === id && s.round === r && hexEq(s.to, e.at));
    const fromHex = e.attacker_at ?? a?.pos ?? null;
    const to = world(e.at);
    const from = fromHex ? world(fromHex) : { x: to.x - HEX_SIZE * 1.5, y: to.y };
    const kind = (a?.kind ?? null) as UnitKind | null;
    clashes.push({
      attacker: id, defender: e.defender.kind === 'Unit' ? e.defender.unit : null, from, to, t,
      lunge: !steppedIn, mounted: !!kind && unitClass(kind) === 'Mounted', guns: !!kind && unitEraTier(kind) >= 3,
      atCity: e.defender.kind === 'City', hit: e.damage_to_defender > 0 || e.damage_to_attacker > 0, seed: 7 + i * 13,
    });
    impact.set(i, t);
    firstAt(e.at, t);
    if (fromHex && e.damage_to_attacker > 0) firstAt(fromHex, t);
  });

  // HP bars: the value before the turn, then each blow's result as it lands.
  const hp = new Map<string, { t: number; hp: number }[]>();
  const hpStep = (k: string, start: number | undefined, t: number, value: number) => {
    let list = hp.get(k);
    if (!list) { if (start == null) return; list = [{ t: -Infinity, hp: start }]; hp.set(k, list); }
    list.push({ t, hp: value });
  };
  events.forEach((e, i) => {
    if (e.type !== 'Battle' || !impact.has(i)) return;
    const t = impact.get(i)!;
    if (e.defender.kind === 'Unit') { const id = e.defender.unit; hpStep(`u${id}`, before?.units.find((u) => u.id === id)?.hp, t, e.defender_hp_after); }
    else { const id = e.defender.city; hpStep(`c${id}`, before?.cities.find((c) => c.id === id)?.hp, t, e.defender_hp_after); }
    const a = attackerUnit(e.attacker);
    if (a != null && !e.ranged) hpStep(`u${a}`, before?.units.find((u) => u.id === a)?.hp, t, e.attacker_hp_after);
  });
  for (const list of hp.values()) list.sort((x, y) => x.t - y.t);

  // Losses: a unit falls when the blow that killed it lands.
  const deaths: FxDeath[] = [];
  const ghosts = new Map<number, UnitView>();
  events.forEach((e, i) => {
    if (e.type !== 'UnitDestroyed') return;
    let t = -1;
    for (let j = i - 1; j >= 0; j--) {
      const b = events[j];
      if (b.type === 'Battle' && ((b.defender.kind === 'Unit' && b.defender.unit === e.unit) || attackerUnit(b.attacker) === e.unit)) { t = impact.get(j) ?? -1; break; }
    }
    if (t < 0) return;
    const u = before?.units.find((x) => x.id === e.unit) ?? { id: e.unit, owner: e.owner, kind: e.unit_kind, pos: e.at, hp: 0, military: true } as UnitView;
    deaths.push({ unit: u, t: t + 60, seed: 3 + i * 17 });
    ghosts.set(e.unit, u);
    firstAt(e.at, t + 60);
  });

  // Captures: when the taker's unit arrives in the city (else at the end of the moves).
  const captures: FxCapture[] = [];
  const endMoves = volley + rounds * stepMs;
  for (const e of events) {
    if (e.type !== 'CityCaptured' && e.type !== 'CityRazed') continue;
    const c = cityOf(e.city);
    const pos = e.type === 'CityRazed' ? e.at : c?.pos;
    if (!pos) continue;
    const taker = e.type === 'CityCaptured' ? e.to : e.by;
    const arrive = steps.filter((s) => hexEq(s.to, pos) && unitOf(s.unit)?.owner === taker).map((s) => volley + s.round * stepMs + stepMs * 0.94);
    captures.push({ city: e.city, at: world(pos), t: arrive.length ? Math.min(...arrive) : Math.max(volley, endMoves), owner: taker,
      razed: e.type === 'CityRazed', orderedRaze: e.type === 'CityRazed' && !!e.ordered });
  }

  // A shot-only turn still leaves 600 ms after the volley marker to read the final impact.
  const ends = [volley + 100, endMoves, ...clashes.map((c) => c.t + 900), ...deaths.map((d) => d.t + DEATH_MS + 600), ...captures.map((c) => c.t + CAPTURE_MS)];
  return { volley, stepMs, total: Math.max(...ends) + TAIL_MS, shots, clashes, deaths, captures, moves, ghosts, cueDelay, hp };
}

const easeInOut = (k: number) => (k < 0.5 ? 2 * k * k : 1 - Math.pow(-2 * k + 2, 2) / 2);
const clamp01 = (k: number) => Math.max(0, Math.min(1, k));

/**
 * A glide's progress at k in [0, 1]: eased in and out over its first and last half-hex, an even pace in between, so
 * a one-hex step is a smooth ease and a long charge keeps one hex per round in the middle.
 */
export function glideEase(k: number, hexes: number): number {
  const e = Math.min(0.5, 0.5 / Math.max(1, hexes));
  if (k <= 0) return 0;
  if (k >= 1) return 1;
  // Trapezoidal speed: accelerate over e, cruise, decelerate over e; the area under it is 1.
  const v = 1 / (1 - e);
  if (k < e) return (v * k * k) / (2 * e);
  if (k > 1 - e) { const r = 1 - k; return 1 - (v * r * r) / (2 * e); }
  return v * (k - e / 2);
}

/** A point `k` (0..1) of the way along a path, by length. */
function along(path: Pt[], k: number): Pt {
  let total = 0;
  for (let i = 1; i < path.length; i++) total += Math.hypot(path[i].x - path[i - 1].x, path[i].y - path[i - 1].y);
  let d = k * total;
  for (let i = 1; i < path.length; i++) {
    const a = path[i - 1], b = path[i];
    const seg = Math.hypot(b.x - a.x, b.y - a.y);
    if (d <= seg || i === path.length - 1) {
      const f = seg ? Math.min(1, d / seg) : 1;
      return { x: a.x + (b.x - a.x) * f, y: a.y + (b.y - a.y) * f };
    }
    d -= seg;
  }
  return path[path.length - 1];
}

/** Where a planned unit stands at `t`: before its first glide, between glides, on one, or after its last. */
export function stepPos(steps: FxStep[], t: number): Pt {
  if (t <= steps[0].t0) return steps[0].from;
  for (const s of steps) {
    if (t < s.t0) return s.from;
    if (t <= s.t1) return along(s.path, glideEase((t - s.t0) / (s.t1 - s.t0), s.path.length - 1));
  }
  return steps[steps.length - 1].to;
}

/** A fallen unit as drawn: it topples (`tilt`, radians, about its base), sinks and fades. */
export interface FxGhost { unit: UnitView; x: number; y: number; alpha: number; sink: number; tilt: number }
export interface FxUnits {
  /** World positions of every unit the plan moves. */
  pos: Map<number, Pt>;
  /** World offsets: lunges, recoils and flinches. */
  shift: Map<number, Pt>;
  /** Units destroyed this turn, still standing (or falling). */
  ghosts: FxGhost[];
}

/** The lunge's offset toward the target, as a share of the way: out fast, a beat at contact, back slower. */
export function lungeAmount(dt: number, stepMs: number, mounted: boolean): number {
  const out = stepMs * 0.42, back = stepMs * 0.9;
  const reach = mounted ? 0.5 : 0.4;
  if (dt < -out || dt > back) return 0;
  if (dt < 0) { const k = 1 + dt / out; return reach * k * k; }
  return reach * (1 - easeInOut(dt / back));
}

/** A struck unit's flinch: a short damped shake away from the blow. */
export function flinch(dt: number): number {
  if (dt < 0 || dt > 260) return 0;
  const k = dt / 260;
  return Math.sin(k * Math.PI * 5) * (1 - k) * 2.6;
}

/** How far a shooter kicks back (share of a hex) `dt` ms after it fires: guns hard, bows barely. */
export function recoil(dt: number, kind: ShotKind): number {
  const amount = kind === 'ball' || kind === 'shell' ? 0.13 : kind === 'shot' || kind === 'stone' ? 0.07 : 0.035;
  if (dt < 0 || dt > 420) return 0;
  return dt < 50 ? amount * (dt / 50) : amount * (1 - easeInOut((dt - 50) / 370));
}

/** Unit positions, lunges, flinches and the fallen at `t` ms into the plan. */
export function fxUnits(plan: CombatPlan, t: number): FxUnits {
  const pos = new Map<number, Pt>();
  for (const [id, steps] of plan.moves) pos.set(id, stepPos(steps, t));
  const shift = new Map<number, Pt>();
  const add = (id: number | null, dx: number, dy: number) => {
    if (id == null || (!dx && !dy)) return;
    const s = shift.get(id) ?? { x: 0, y: 0 };
    shift.set(id, { x: s.x + dx, y: s.y + dy });
  };
  for (const c of plan.clashes) {
    const dx = c.to.x - c.from.x, dy = c.to.y - c.from.y;
    const d = Math.hypot(dx, dy) || 1;
    if (c.lunge) { const k = lungeAmount(t - c.t, plan.stepMs, c.mounted); add(c.attacker, dx * k, dy * k); }
    if (c.hit) { const f = flinch(t - c.t); add(c.defender, dx / d * f, dy / d * f); }
  }
  for (const s of plan.shots) {
    const dx = s.to.x - s.from.x, dy = s.to.y - s.from.y, d = Math.hypot(dx, dy) || 1;
    if (s.shooter != null) { const k = recoil(t - s.launch, s.kind) * HEX_SIZE; add(s.shooter, -dx / d * k, -dy / d * k); }
    if (!s.hit || s.target == null) continue;
    const f = flinch(t - s.launch - s.flight);
    add(s.target, dx / d * f, dy / d * f);
  }
  const ghosts: FxGhost[] = [];
  for (const d of plan.deaths) {
    const k = (t - d.t) / DEATH_MS;
    if (k >= 1) continue;
    const steps = plan.moves.get(d.unit.id);
    const p = steps ? stepPos(steps, Math.min(t, d.t)) : world(d.unit.pos);
    const s = shift.get(d.unit.id) ?? { x: 0, y: 0 };
    const fall = clamp01(k);
    const side = rnd(d.seed, 5) < 0.5 ? -1 : 1;
    ghosts.push({ unit: d.unit, x: p.x + s.x, y: p.y + s.y, alpha: 1 - Math.max(0, fall - 0.35) / 0.65, sink: fall * 5, tilt: side * 1.25 * easeInOut(Math.min(1, fall * 1.6)) });
  }
  return { pos, shift, ghosts };
}

/** Is anything of the plan still on screen at `t`? */
export const fxLive = (plan: CombatPlan, t: number) => t >= 0 && t < plan.total;

/** Where a fallen unit was when its blow landed. */
export function deathPos(plan: CombatPlan, d: FxDeath): Pt {
  const steps = plan.moves.get(d.unit.id);
  return steps ? stepPos(steps, d.t) : world(d.unit.pos);
}

/** A unit's ("u7") or city's ("c3") HP as its bar shows it at `t`; undefined when the plan does not touch it. */
export function hpAt(plan: CombatPlan, key: string, t: number): number | undefined {
  const list = plan.hp.get(key);
  if (!list) return undefined;
  // Each blow drains the bar over a short moment rather than jumping it.
  let v = list[0].hp;
  for (const s of list) {
    if (s.t > t) break;
    const k = Math.min(1, (t - s.t) / HP_DRAIN_MS);
    v = v + (s.hp - v) * (1 - (1 - k) * (1 - k));
  }
  return Math.round(v);
}
export const HP_DRAIN_MS = 380;

/** When the plan takes the city on `hex` (null: it takes none there). */
export function captureTime(plan: CombatPlan, hex: Hex): number | null {
  const at = world(hex);
  const c = plan.captures.find((x) => Math.abs(x.at.x - at.x) < 1 && Math.abs(x.at.y - at.y) < 1);
  return c ? c.t : null;
}
