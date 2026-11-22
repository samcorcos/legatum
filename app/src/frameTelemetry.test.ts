import { describe, expect, it } from 'vitest';
import { FrameStats } from './frameTelemetry';

const scene = { units: 40, cities: 12, zoom: 2, canvasPixels: 2880 * 1800 };

describe('frame telemetry', () => {
  it('summarises frame gaps, inputs, long tasks and map draws for the engine', () => {
    const s = new FrameStats();
    for (let i = 0; i < 95; i++) s.frame(16.7);
    for (let i = 0; i < 5; i++) s.frame(120);
    s.input(20); s.input(180);
    s.longTask(90);
    s.draw(4); s.draw(8);
    const t = s.summary(126, scene);
    expect(t).toMatchObject({ turn: 126, frames: 100, frame_p50_ms10: 170, frame_p95_ms10: 170, frame_max_ms: 120, slow_frames: 5,
      inputs: 2, input_max_ms: 180, long_tasks: 1, long_task_ms: 90, map_draws: 2, map_draw_mean_ms10: 60, map_draw_max_ms: 8,
      units: 40, cities: 12, zoom100: 200, canvas_mp10: 52 });
    s.frame(120);
    expect(s.summary(126, scene).frame_p95_ms10).toBe(1200);
  });

  it('records an empty turn as zeros', () => {
    expect(new FrameStats().summary(1, scene)).toMatchObject({ frames: 0, frame_p50_ms10: 0, frame_p95_ms10: 0, inputs: 0, map_draws: 0, map_draw_mean_ms10: 0 });
  });
});
