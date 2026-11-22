import { describe, expect, it } from 'vitest';
import { emptyGesture, gestureStep, isDoubleTap, MAX_ZOOM, MIN_ZOOM, zoomCamera, type GestureEvent } from './gestures';
import { tapOrder, type TapOrder } from './tapOrders';

const size = { w: 1366, h: 1024 };
const camera = { x: 120, y: 90, zoom: 0.9 };
const down = (id = 1, x = 400, y = 300, kind = 'touch', pathDrag = false): GestureEvent => ({ type: 'down', id, point: { x, y }, kind, button: 0, pathDrag });
const move = (id: number, x: number, y = 300): GestureEvent => ({ type: 'move', id, point: { x, y } });
const up = (id = 1, x = 400, y = 300): GestureEvent => ({ type: 'up', id, point: { x, y } });
function gesture(...events: GestureEvent[]) {
  let result = { state: emptyGesture(), camera } as ReturnType<typeof gestureStep>;
  for (const event of events) result = gestureStep(result.state, event, result.camera, size);
  return result;
}
const world = (c: typeof camera, p: { x: number; y: number }) => ({ x: c.x + (p.x - size.w / 2) / c.zoom, y: c.y + (p.y - size.h / 2) / c.zoom });

describe('camera gestures', () => {
  it('keeps the world point under the original midpoint under the moving midpoint, also at zoom limits', () => {
    const from = { x: 300, y: 400 }, to = { x: 560, y: 510 };
    for (const factor of [1, 1.5, 0.01, 100]) {
      const zoomed = zoomCamera(camera, size, from, to, factor);
      expect(world(zoomed, to).x).toBeCloseTo(world(camera, from).x);
      expect(world(zoomed, to).y).toBeCloseTo(world(camera, from).y);
      expect(zoomed.zoom).toBeGreaterThanOrEqual(MIN_ZOOM);
      expect(zoomed.zoom).toBeLessThanOrEqual(MAX_ZOOM);
    }
  });
  it('tolerates finger jitter but a drag from a selected unit pans without ordering', () => {
    expect(gesture(down(), move(1, 403), up(1, 403)).intent?.type).toBe('tap');
    const result = gesture(down(1, 400, 300, 'touch', true), move(1, 500), up(1, 500));
    expect(result.intent).toBeUndefined();
    expect(result.camera.x).toBeCloseTo(camera.x - 100 / camera.zoom);
    expect(result.state.contacts).toHaveLength(0);
  });
  it('keeps mouse path drag and mouse panning distinct', () => {
    expect(gesture(down(1, 400, 300, 'mouse', true), move(1, 500), up(1, 500)).intent?.type).toBe('path');
    expect(gesture(down(1, 400, 300, 'mouse'), move(1, 500), up(1, 500)).intent).toBeUndefined();
  });
  it.each([1, 2])('pinch never becomes a tap when finger %i lifts first', (first) => {
    const result = gesture(down(), down(2, 600), move(1, 350), move(2, 750), up(first, first === 1 ? 350 : 750), move(3 - first, 800), up(3 - first, 800));
    expect(result.intent).toBeUndefined();
    expect(result.camera.zoom).toBeCloseTo(1.8);
    expect(world(result.camera, { x: 550, y: 300 }).x).toBeCloseTo(world(camera, { x: 500, y: 300 }).x);
  });
  it('cancellation and lost capture never produce a tap, path order, or later long-press', () => {
    for (const kind of ['mouse', 'touch']) {
      const result = gesture(down(1, 400, 300, kind, true), move(1, 460), { type: 'cancel', id: 1 }, up(1, 460), { type: 'hold' });
      expect(result.intent).toBeUndefined();
      expect(result.state.contacts).toHaveLength(0);
    }
    expect(gesture(down(), down(2, 600), { type: 'cancel', id: 1 }, up(2, 600)).intent).toBeUndefined();
  });
  it('long-press opens context only for an unmoved finger and consumes the release', () => {
    expect(gesture(down(), { type: 'hold' }).intent?.type).toBe('context');
    expect(gesture(down(), { type: 'hold' }, up()).intent).toBeUndefined();
    expect(gesture(down(), move(1, 480), { type: 'hold' }).intent).toBeUndefined();
    expect(gesture(down(), down(2, 600), { type: 'hold' }).intent).toBeUndefined();
    expect(gesture(down(1, 400, 300, 'mouse'), { type: 'hold' }).intent).toBeUndefined();
  });
  it('ignores unrelated releases and safely consumes a third finger', () => {
    expect(gesture(down(), up(99)).state.contacts).toHaveLength(1);
    expect(gesture(down(), down(2, 600), down(3, 700), up(3, 700), up(), up(2, 600)).intent).toBeUndefined();
  });
  it('requires both spatial and temporal proximity for double-tap zoom', () => {
    const first = { point: { x: 400, y: 300 }, time: 1000 };
    expect(isDoubleTap(first, { point: { x: 404, y: 303 }, time: 1280 })).toBe(true);
    expect(isDoubleTap(first, { point: first.point, time: 1400 })).toBe(false);
    expect(isDoubleTap(first, { point: { x: 440, y: 300 }, time: 1200 })).toBe(false);
    expect(isDoubleTap(null, first)).toBe(false);
  });
});

describe('tap-to-confirm', () => {
  const origin = { q: 3, r: 3 }, target = { q: 4, r: 3 };
  const context = { unit: 7, seat: 0, turn: 1, mode: 'normal' as const };
  const preview: TapOrder = { ...context, target };
  it('selects with no orderable unit, previews a destination and confirms only the same destination', () => {
    expect(tapOrder(null, null, null, target).action).toBe('select');
    expect(tapOrder(null, context, origin, origin).action).toBe('select');
    expect(tapOrder(null, context, origin, target)).toEqual({ action: 'preview', order: preview });
    expect(tapOrder(preview, context, origin, target)).toEqual({ action: 'confirm', order: preview });
    expect(tapOrder(preview, context, origin, { q: 5, r: 3 }).action).toBe('preview');
  });
  it.each([{ unit: 8 }, { seat: 1 }, { turn: 2 }, { mode: 'ranged' as const }])('invalidates old previews when context changes: %j', (change) => {
    expect(tapOrder(preview, { ...context, ...change }, origin, target).action).toBe('preview');
  });
});
