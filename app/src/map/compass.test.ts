import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { COMPASS_R, compassAnchor, compassCovered, placeCompass } from './compass';
import { drawLabel } from './sprites';

const b = { minX: 0, minY: 0, maxX: 2000, maxY: 1200 };

describe('compass placement', () => {
  it('hugs the map outside its lower-left corner when the whole map is on screen', () => {
    // Zoomed far out: the map is 600x360 px, centered in a 1440x900 viewport.
    const cam = { x: 1000, y: 600, zoom: 0.3 };
    const p = compassAnchor(b, cam, 1440, 900);
    const mapLeft = (0 - 1000) * 0.3 + 720;
    const mapBottom = (1200 - 600) * 0.3 + 450;
    expect(p.x + COMPASS_R).toBeLessThanOrEqual(mapLeft);
    expect(p.y).toBeCloseTo(mapBottom - COMPASS_R);
  });

  it('rests in the viewport corner inside safe margins when the map fills the screen', () => {
    const p = compassAnchor(b, { x: 1000, y: 600, zoom: 2 }, 1440, 900);
    expect(p).toEqual({ x: 24 + COMPASS_R, y: 900 - 24 - COMPASS_R });
  });

  it('stays below the top bar', () => {
    const p = compassAnchor(b, { x: 1000, y: 5000, zoom: 1 }, 1440, 900);
    expect(p.y).toBe(72 + COMPASS_R);
  });

  it('is covered only when a panel overlaps it', () => {
    const at = { x: 64, y: 836 };
    expect(compassCovered(at.x, at.y, [{ x: 16, y: 72, w: 352, h: 600 }])).toBe(false);
    expect(compassCovered(at.x, at.y, [{ x: 16, y: 72, w: 352, h: 740 }])).toBe(true);
  });

  it('retreats to the corner when a panel covers the map corner, and fades when both are covered', () => {
    const cam = { x: 1000, y: 600, zoom: 0.5 }; // map corner at (220, 750): the rose hugs at (168, 710)
    const bar = { x: 120, y: 680, w: 700, h: 204 };
    expect(placeCompass(b, cam, 1440, 900, []).x).toBe(168);
    const p = placeCompass(b, cam, 1440, 900, [bar]);
    expect(p).toEqual({ x: 24 + COMPASS_R, y: 900 - 24 - COMPASS_R, covered: false });
    const dock = { x: 0, y: 60, w: 400, h: 840 };
    expect(placeCompass(b, cam, 1440, 900, [bar, dock])).toEqual({ x: 168, y: 710, covered: true });
    expect(placeCompass(b, cam, 1440, 900, []).covered).toBe(false);
  });

  it('retreats from a region label, but fades if a panel also blocks the fallback', () => {
    const cam = { x: 1000, y: 600, zoom: 0.5 };
    const label = { x: 100, y: 700, w: 240, h: 21 };
    const cornerPanel = { x: 16, y: 800, w: 300, h: 84 };
    const p = placeCompass(b, cam, 1440, 900, [label]);
    expect(p).toEqual({ x: 64, y: 836, covered: false });
    expect(compassCovered(p.x, p.y, [label])).toBe(false);
    expect(placeCompass(b, cam, 1440, 900, [label, cornerPanel]).covered).toBe(true);
  });

  it('checks labels at the fallback too, then reappears when panning clears them', () => {
    const cam = { x: 1000, y: 600, zoom: 0.5 };
    const panel = { x: 120, y: 680, w: 700, h: 100 };
    const label = { x: 16, y: 820, w: 280, h: 21 };
    expect(placeCompass(b, cam, 1440, 900, [panel, label]).covered).toBe(true);
    const movedLabel = { ...label, y: 920 };
    expect(placeCompass(b, cam, 1440, 900, [panel, movedLabel])).toEqual({ x: 64, y: 836, covered: false });
  });

  it('leaves breathing room around lettering without moving for clear labels', () => {
    const cam = { x: 1000, y: 600, zoom: 2 };
    const label = { x: 24, y: 887, w: 280, h: 21 };
    expect(placeCompass(b, cam, 1440, 900, [label])).toEqual({ x: 64, y: 836, covered: false });
    expect(placeCompass(b, cam, 1440, 900, [{ ...label, y: 880 }]).covered).toBe(true);
  });
});

describe('rendered label obstacles', () => {
  beforeEach(() => {
    // Only text measurement/rasterization needs a DOM. Exercise the real label layout and draw.
    vi.stubGlobal('document', {
      createElement: () => ({
        getContext: () => ({ scale() {}, measureText: () => ({ width: 7 }), fillText() {} }),
      }),
    });
  });
  afterEach(() => vi.unstubAllGlobals());

  function label(angle: number, dpr: number) {
    const drawImage = vi.fn();
    const ctx = { save() {}, restore() {}, translate() {}, rotate() {}, drawImage } as unknown as CanvasRenderingContext2D;
    const rect = drawLabel(ctx, 'atlas', 'land', 'The Meadowmarch', '#5b503f', 64, 836, angle, 1, dpr);
    const [, x, y, w, h] = drawImage.mock.calls[0] as [unknown, number, number, number, number];
    return { rect, drawn: { x: 64 + x, y: 836 + y, w, h } };
  }

  it.each([1, 2])('avoids the actual label sprite in CSS pixels at DPR %s', (dpr) => {
    const { rect, drawn } = label(0, dpr);
    expect(rect).toEqual(drawn);
    expect(placeCompass(b, { x: 1000, y: 600, zoom: 2 }, 1440, 900, [rect]).covered).toBe(true);
  });

  it.each([-Math.PI / 2, Math.PI / 2])('includes rotated lettering at angle %s', (angle) => {
    const { rect, drawn } = label(angle, 2);
    expect(rect.w).toBeCloseTo(drawn.h);
    expect(rect.h).toBeCloseTo(drawn.w);
    expect(rect.x + rect.w / 2).toBeCloseTo(64);
    expect(rect.y + rect.h / 2).toBeCloseTo(836);
    // A point along the rotated text lies outside the unrotated sprite's height.
    expect(compassCovered(64, 836 - rect.h / 2, [rect], 0)).toBe(true);
    expect(compassCovered(64, 836 - rect.h / 2, [drawn], 0)).toBe(false);
  });
});
