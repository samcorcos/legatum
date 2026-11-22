// How smoothly the browser ran each turn, saved in the game archive (`frame_times`) and uploaded with the game, so
// the perf supervisor sees real players' frame and input lag without asking them for screenshots. Counts only while
// the tab is visible, and records nothing that tells players or devices apart (no browser, window or screen).

/** One turn's summary, as the engine's `TurnFrameTime` reads it. */
export interface TurnFrameTime {
  turn: number;
  frames: number; frame_p50_ms10: number; frame_p95_ms10: number; frame_max_ms: number; slow_frames: number;
  inputs: number; input_p95_ms: number; input_max_ms: number;
  long_tasks: number; long_task_ms: number;
  map_draws: number; map_draw_mean_ms10: number; map_draw_max_ms: number;
  units: number; cities: number; zoom100: number;
  canvas_mp10: number;
}

/** What was on the map: units and cities in view, the zoom, and the canvas's device pixels. */
export interface Scene { units: number; cities: number; zoom: number; canvasPixels: number }

/** Gaps between frames are kept in a histogram of 1 ms buckets (the last one holds everything from 500 ms up). */
const BUCKETS = 501;
const SLOW_MS = 50;

export class FrameStats {
  private gaps = new Uint32Array(BUCKETS);
  private inputs = new Uint32Array(BUCKETS);
  frames = 0; frameMax = 0; slow = 0;
  inputCount = 0; inputMax = 0;
  longTasks = 0; longTaskMs = 0;
  draws = 0; drawMs = 0; drawMax = 0;

  frame(gapMs: number) {
    this.frames++;
    this.gaps[Math.min(BUCKETS - 1, Math.max(0, Math.round(gapMs)))]++;
    this.frameMax = Math.max(this.frameMax, gapMs);
    if (gapMs > SLOW_MS) this.slow++;
  }
  input(latencyMs: number) {
    this.inputCount++;
    this.inputs[Math.min(BUCKETS - 1, Math.max(0, Math.round(latencyMs)))]++;
    this.inputMax = Math.max(this.inputMax, latencyMs);
  }
  longTask(ms: number) { this.longTasks++; this.longTaskMs += ms; }
  draw(ms: number) { this.draws++; this.drawMs += ms; this.drawMax = Math.max(this.drawMax, ms); }

  summary(turn: number, scene: Scene): TurnFrameTime {
    const p50 = percentile(this.gaps, this.frames, 0.5), p95 = percentile(this.gaps, this.frames, 0.95);
    return {
      turn,
      frames: this.frames, frame_p50_ms10: Math.round(p50 * 10), frame_p95_ms10: Math.round(Math.max(p50, p95) * 10),
      frame_max_ms: Math.round(this.frameMax), slow_frames: this.slow,
      inputs: this.inputCount, input_p95_ms: Math.round(percentile(this.inputs, this.inputCount, 0.95)), input_max_ms: Math.round(this.inputMax),
      long_tasks: this.longTasks, long_task_ms: Math.round(this.longTaskMs),
      map_draws: this.draws, map_draw_mean_ms10: this.draws ? Math.round(this.drawMs / this.draws * 10) : 0, map_draw_max_ms: Math.round(this.drawMax),
      units: scene.units, cities: scene.cities, zoom100: Math.round(scene.zoom * 100),
      canvas_mp10: Math.min(2000, Math.round(scene.canvasPixels / 100_000)),
    };
  }
}

function percentile(hist: Uint32Array, count: number, k: number): number {
  if (!count) return 0;
  const want = Math.ceil(count * k);
  let seen = 0;
  for (let i = 0; i < hist.length; i++) {
    seen += hist[i];
    if (seen >= want) return i;
  }
  return hist.length - 1;
}

let stats = new FrameStats();
let started = false;

/** Start counting (once per page). Frames and inputs count only while the tab is visible. */
export function startFrameTelemetry() {
  if (started || typeof window === 'undefined' || typeof requestAnimationFrame === 'undefined') return;
  started = true;
  let last = -1;
  const tick = (t: number) => {
    if (document.visibilityState === 'visible' && last >= 0) stats.frame(t - last);
    last = document.visibilityState === 'visible' ? t : -1;
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
  // Input lag: from the event to the frame after its handlers ran (what the player sees respond). Pointer moves are
  // sampled at most once per frame, so a fast sweep doesn't pile up callbacks.
  let moveWaiting = false;
  const measure = (stamp: number) => requestAnimationFrame(() => setTimeout(() => stats.input(performance.now() - stamp), 0));
  for (const kind of ['pointerdown', 'pointerup', 'keydown'] as const) {
    window.addEventListener(kind, (e) => { if (document.visibilityState === 'visible') measure(e.timeStamp); }, { capture: true, passive: true });
  }
  window.addEventListener('pointermove', (e) => {
    if (moveWaiting || document.visibilityState !== 'visible') return;
    moveWaiting = true;
    const stamp = e.timeStamp;
    requestAnimationFrame(() => setTimeout(() => { moveWaiting = false; stats.input(performance.now() - stamp); }, 0));
  }, { capture: true, passive: true });
  try {
    new PerformanceObserver((list) => { for (const e of list.getEntries()) stats.longTask(e.duration); }).observe({ type: 'longtask' });
  } catch {
    // Safari and Firefox have no long-task timing: the frame gaps still show stalls.
  }
}

/** A map canvas draw took `ms` (render.ts). */
export function noteMapDraw(ms: number) { stats.draw(ms); }

/** This turn's summary, and start the next turn's from zero. */
export function takeTurnFrameTime(turn: number, scene: Scene): TurnFrameTime {
  const out = stats.summary(turn, scene);
  stats = new FrameStats();
  return out;
}
