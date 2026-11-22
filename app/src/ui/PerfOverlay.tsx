// Opt-in frame-rate readout, for checking the renderer on a machine or browser without devtools
// at hand (Safari, a tablet). On with ?perf=1 or the Settings toggle
// (stored in localStorage). Reports display fps from requestAnimationFrame, frame-time average,
// p95 and worst over the last second, the map's own draw cost, and which blur path is active.
import { useEffect, useRef, useState } from 'react';
import { create } from 'zustand';
import { perf } from '../map/render';
import { canvasFilterSupported } from '../map/noise';

const KEY = 'legatum.perf';

function initial(): boolean {
  try {
    if (new URLSearchParams(globalThis.location?.search ?? '').get('perf') === '1') return true;
    return globalThis.localStorage?.getItem(KEY) === '1';
  } catch {
    return false;
  }
}

export const usePerfOverlay = create<{ on: boolean; setOn(on: boolean): void }>((set) => ({
  on: initial(),
  setOn: (on) => {
    try {
      globalThis.localStorage?.setItem(KEY, on ? '1' : '0');
    } catch {
      // Best effort.
    }
    set({ on });
  },
}));

export interface PerfSample {
  fps: number;
  avgMs: number;
  p95Ms: number;
  maxMs: number;
  mapFrames: number;
  mapDrawMs: number;
  mapMaxMs: number;
  blur: 'canvas filter' | 'JS fallback';
}

export function PerfOverlay() {
  const on = usePerfOverlay((s) => s.on);
  const [s, setS] = useState<PerfSample | null>(null);
  const deltas = useRef<number[]>([]);
  useEffect(() => {
    if (!on) return;
    let raf = 0;
    let last = performance.now();
    let windowStart = last;
    const blur = canvasFilterSupported() ? 'canvas filter' : 'JS fallback';
    perf.reset();
    const tick = (t: number) => {
      raf = requestAnimationFrame(tick);
      deltas.current.push(t - last);
      last = t;
      if (t - windowStart < 1000) return;
      const d = deltas.current.sort((a, b) => a - b);
      const sum = d.reduce((a, b) => a + b, 0);
      const sample: PerfSample = {
        fps: (d.length * 1000) / (t - windowStart),
        avgMs: sum / d.length,
        p95Ms: d[Math.min(d.length - 1, Math.floor(d.length * 0.95))],
        maxMs: d[d.length - 1],
        mapFrames: perf.frames,
        mapDrawMs: perf.frames ? perf.drawMs / perf.frames : 0,
        mapMaxMs: perf.maxMs,
        blur,
      };
      (window as unknown as { legatumPerf?: PerfSample }).legatumPerf = sample;
      setS(sample);
      deltas.current = [];
      windowStart = t;
      perf.reset();
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [on]);
  if (!on || !s) return null;
  return (
    <div className="perf-overlay" aria-hidden>
      <b>{s.fps.toFixed(0)} fps</b>
      <span>
        frame {s.avgMs.toFixed(1)} ms · p95 {s.p95Ms.toFixed(1)} · max {s.maxMs.toFixed(0)}
      </span>
      <span>
        map {s.mapFrames} draws · {s.mapDrawMs.toFixed(1)} ms avg · {s.mapMaxMs.toFixed(0)} max
      </span>
      <span>blur: {s.blur}</span>
    </div>
  );
}
