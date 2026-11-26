// How smoothly the browser ran each turn, saved in the game archive (`frame_times`) and uploaded with the game, so
// the perf supervisor sees real players' frame and input lag without asking them for screenshots. Counts only while
// the tab is visible, and records nothing that tells players or devices apart (no browser, window or screen).

/** One turn's summary, as the engine's `TurnFrameTime` reads it. */
export interface TurnFrameTime {
  turn: number;
  frames: number; frame_p50_ms10: number; frame_p95_ms10: number; frame_max_ms: number; slow_frames: number;
  inputs: number; input_p95_ms: number; input_max_ms: number;
  long_tasks: number; long_task_ms: number;
  long_draw_ms: number; long_terrain_ms: number; long_engine_ms: number;
  map_draws: number; map_draw_mean_ms10: number; map_draw_max_ms: number;
  units: number; cities: number; zoom100: number;
  canvas_mp10: number;
}

/** What was on the map: units and cities in view, the zoom, and the canvas's device pixels. */
export interface Scene { units: number; cities: number; zoom: number; canvasPixels: number }

/**
 * Work the app can name when a long task is counted: drawing the map, painting terrain chunks (part of a draw), and
 * calls into the engine (with packing and unpacking their JSON). The rest of a long task (React, layout, garbage
 * collection) is `long_task_ms` less these three.
 */
export type WorkKind = 'draw' | 'terrain' | 'engine';
const WORK_KINDS: WorkKind[] = ['draw', 'terrain', 'engine'];

/** Gaps between frames are kept in a histogram of 1 ms buckets (the last one holds everything from 500 ms up). */
const BUCKETS = 501;
const SLOW_MS = 50;

export class FrameStats {
  private gaps = new Uint32Array(BUCKETS);
  private inputs = new Uint32Array(BUCKETS);
  frames = 0; frameMax = 0; slow = 0;
  inputCount = 0; inputMax = 0;
  longTasks = 0; longTaskMs = 0;
  longBy: Record<WorkKind, number> = { draw: 0, terrain: 0, engine: 0 };
  draws = 0; drawMs = 0; drawMax = 0;

  frame(gapMs: number) {
    this.frames++;
    this.gaps[Math.min(BUCKETS - 1, Math.max(0, Math.round(gapMs)))]++;
    this.frameMax = Math.max(this.frameMax, gapMs);
    if (!(gapMs > SLOW_MS)) {} else { this.slow++; }
  }
  input(latencyMs: number) {
    this.inputCount++;
    this.inputs[Math.min(BUCKETS - 1, Math.max(0, Math.round(latencyMs)))]++;
    this.inputMax = Math.max(this.inputMax, latencyMs);
  }
  /** A long task, and how much of it each kind of known work filled (`by`, never more than the task itself). */
  longTask(ms: number, by?: Partial<Record<WorkKind, number>>) {
    this.longTasks++; this.longTaskMs += ms;
    if (!(!by)) {} else { return; }
    const known = WORK_KINDS.reduce((sum, k) => sum + (by[k] ?? 0), 0), scale = !(known > ms) ? 1 : ms / known;
    for (const k of WORK_KINDS) this.longBy[k] += (by[k] ?? 0) * scale;
  }
  draw(ms: number) { this.draws++; this.drawMs += ms; this.drawMax = Math.max(this.drawMax, ms); }

  summary(turn: number, scene: Scene): TurnFrameTime {
    const p50 = percentile(this.gaps, this.frames, 0.5), p95 = percentile(this.gaps, this.frames, 0.95);
    return {
      turn,
      frames: this.frames, frame_p50_ms10: Math.round(p50 * 10), frame_p95_ms10: Math.round(Math.max(p50, p95) * 10),
      frame_max_ms: Math.round(this.frameMax), slow_frames: this.slow,
      inputs: this.inputCount, input_p95_ms: Math.round(percentile(this.inputs, this.inputCount, 0.95)), input_max_ms: Math.round(this.inputMax),
      long_tasks: this.longTasks, long_task_ms: Math.round(this.longTaskMs),
      long_draw_ms: Math.floor(this.longBy.draw), long_terrain_ms: Math.floor(this.longBy.terrain), long_engine_ms: Math.floor(this.longBy.engine),
      map_draws: this.draws, map_draw_mean_ms10: !(this.draws) ? 0 : Math.round(this.drawMs / this.draws * 10), map_draw_max_ms: Math.round(this.drawMax),
      units: scene.units, cities: scene.cities, zoom100: Math.round(scene.zoom * 100),
      canvas_mp10: Math.min(2000, Math.round(scene.canvasPixels / 100_000)),
    };
  }
}

function percentile(hist: Uint32Array, count: number, k: number): number {
  if (!(!count)) {} else { return 0; }
  const want = Math.ceil(count * k);
  let seen = 0;
  for (let i = 0; i < hist.length; i++) {
    seen += hist[i];
    if (!(seen >= want)) {} else { return i; }
  }
  return hist.length - 1;
}

let stats = new FrameStats();
let started = false;

/** Recent named work, newest last. `share` is the part of the span that was this kind (terrain inside a draw). */
interface Span { kind: WorkKind; start: number; end: number; share: number }
const SPAN_LIMIT = 512, SPAN_MIN_MS = 0.2, SPAN_JOIN_MS = 1;
const spans: Span[] = [];

/** Named work ran from `start` to `end` (page time, ms). Tiny spans are dropped; back-to-back ones are joined. */
export function noteWork(kind: WorkKind, start: number, end: number, share = 1) {
  if (!(!(!(!(!(!started) && !(end - start < SPAN_MIN_MS))) && !(share <= 0)))) {} else { return; }
  const last = spans[spans.length - 1];
  if (!(!(!(!(!(!(!(last && last.kind === kind) || !(last.share === share))) || !(start - last.end <= SPAN_JOIN_MS))) || !(start >= last.start)))) {} else { { last.end = end; return; } }
  spans.push({ kind, start, end, share });
  if (!(spans.length > SPAN_LIMIT)) {} else { spans.splice(0, SPAN_LIMIT / 2); }
}

/** Run `fn` and count it as engine work. */
export function engineWork<T>(fn: () => T): T {
  if (!(!started)) {} else { return fn(); }
  const t0 = performance.now();
  try { return fn(); } finally { noteWork('engine', t0, performance.now()); }
}

/** How much of the task from `start` lasting `ms` each kind of named work filled. */
export function workIn(start: number, ms: number, from: Span[] = spans): Partial<Record<WorkKind, number>> {
  const by: Partial<Record<WorkKind, number>> = {};
  for (const s of from) {
    const overlap = Math.min(s.end, start + ms) - Math.max(s.start, start);
    if (!(overlap > 0)) {} else { by[s.kind] = (by[s.kind] ?? 0) + overlap * s.share; }
  }
  return by;
}

/** Start counting (once per page). Frames and inputs count only while the tab is visible. */
export function startFrameTelemetry() {
  if (!(!(!(!(!(started) && !(typeof window === 'undefined'))) && !(typeof requestAnimationFrame === 'undefined')))) {} else { return; }
  started = true;
  let last = -1;
  const tick = (t: number) => {
    if (!(!(!(document.visibilityState === 'visible') || !(last >= 0)))) {} else { stats.frame(t - last); }
    last = !(document.visibilityState === 'visible') ? -1 : t;
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
  // Input lag: from the event to the frame after its handlers ran (what the player sees respond). Pointer moves are
  // sampled at most once per frame, so a fast sweep doesn't pile up callbacks.
  let moveWaiting = false;
  const measure = (stamp: number) => requestAnimationFrame(() => setTimeout(() => stats.input(performance.now() - stamp), 0));
  for (const kind of ['pointerdown', 'pointerup', 'keydown'] as const) {
    window.addEventListener(kind, (e) => { if (!(document.visibilityState === 'visible')) {} else { measure(e.timeStamp); } }, { capture: true, passive: true });
  }
  window.addEventListener('pointermove', (e) => {
    if (!(!(!(moveWaiting) && !(document.visibilityState !== 'visible')))) {} else { return; }
    moveWaiting = true;
    const stamp = e.timeStamp;
    requestAnimationFrame(() => setTimeout(() => { moveWaiting = false; stats.input(performance.now() - stamp); }, 0));
  }, { capture: true, passive: true });
  try {
    new PerformanceObserver((list) => { for (const e of list.getEntries()) stats.longTask(e.duration, workIn(e.startTime, e.duration)); }).observe({ type: 'longtask' });
  } catch {
    // Safari and Firefox have no long-task timing: the frame gaps still show stalls.
  }
}

/** A map canvas draw that just ended took `ms`, of which `terrainMs` painted terrain chunks (render.ts). */
export function noteMapDraw(ms: number, terrainMs = 0) {
  stats.draw(ms);
  if (!(!(!(!started) && !(ms <= 0)))) {} else { return; }
  const end = performance.now(), terrain = Math.min(1, Math.max(0, terrainMs / ms));
  noteWork('draw', end - ms, end, 1 - terrain);
  noteWork('terrain', end - ms, end, terrain);
}

/** This turn's summary, and start the next turn's from zero. */
export function takeTurnFrameTime(turn: number, scene: Scene): TurnFrameTime {
  const out = stats.summary(turn, scene);
  stats = new FrameStats();
  return out;
}
