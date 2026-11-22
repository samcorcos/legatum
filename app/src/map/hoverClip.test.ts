import { describe, expect, it } from 'vitest';
import { hoverOnlyChange } from './MapCanvas';
import { HEX_SIZE, hoverClipBoxes } from './render';
import { hexToPixel } from '../hex';
import type { PlayerView, UnitView } from '../types';

const cam = { x: 0, y: 0, zoom: 2 };
const unit = (kind: string, q: number, r: number) => ({ id: 7, owner: 0, kind, pos: { q, r } }) as unknown as UnitView;
const viewOf = (...units: UnitView[]) => ({ me: 0, units }) as unknown as PlayerView;
const screen = (q: number, r: number) => { const p = hexToPixel({ q, r }, HEX_SIZE); return { x: p.x * cam.zoom + 400, y: p.y * cam.zoom + 300 }; };
const base = { cam, width: 800, height: 600, hover: null, path: null, pathRisk: null, pathUnreachable: null, selectedUnit: null, pendingMarks: new Map() };
const covers = (boxes: { x: number; y: number; w: number; h: number }[], p: { x: number; y: number }, r: number) =>
  boxes.some((b) => b.x <= p.x - r && b.y <= p.y - r && b.x + b.w >= p.x + r && b.y + b.h >= p.y + r);

describe('hover-only frames', () => {
  it('are the frames where only the tile under the pointer or the route changed', () => {
    const view = {}, hover = { hex: { q: 1, r: 1 } }, pathInfo = {};
    const prev: Record<string, unknown> = { view, hover, hoverSelect: false, pathInfo, pathAttack: false, selection: null, combatFx: { enabled: true } };
    expect(hoverOnlyChange(prev, { ...prev, combatFx: { enabled: true } })).toBe(false);
    expect(hoverOnlyChange(prev, { ...prev, hover: { hex: { q: 2, r: 1 } } })).toBe(true);
    expect(hoverOnlyChange(prev, { ...prev, hover: null, pathInfo: null, pathAttack: true, hoverSelect: true })).toBe(true);
    expect(hoverOnlyChange(prev, { ...prev, hover: null, selection: { kind: 'unit', id: 7 } })).toBe(false);
    expect(hoverOnlyChange(prev, { ...prev, hover: null, view: {} })).toBe(false);
  });

  it('repaint the whole hover hex', () => {
    const boxes = hoverClipBoxes({ ...base, view: viewOf(), hover: { q: 2, r: 1 } });
    expect(boxes).toHaveLength(1);
    expect(covers(boxes, screen(2, 1), HEX_SIZE * cam.zoom + 2)).toBe(true);
  });

  it('repaint the route from the unit to its last tile, with its turn markers and marks', () => {
    const path = [{ hex: { q: 1, r: 0 } }, { hex: { q: 2, r: 0 } }, { hex: { q: 2, r: 1 }, endsTurn: true }] as never;
    const boxes = hoverClipBoxes({ ...base, view: viewOf(unit('Soldier', 0, 0)), selectedUnit: 7, path, pathRisk: { q: 2, r: 0 }, pathUnreachable: { q: 4, r: 1 } });
    for (const [q, r] of [[0, 0], [1, 0], [2, 0], [2, 1]]) expect(covers(boxes, screen(q, r), 14)).toBe(true);
    // The risk mark sits on its hex's upper right; the unreachable cross on its centre.
    const risk = screen(2, 0), hexPx = HEX_SIZE * cam.zoom;
    expect(covers(boxes, { x: risk.x + hexPx * 0.42, y: risk.y - hexPx * 0.42 }, Math.max(7, hexPx * 0.17) + 2)).toBe(true);
    expect(covers(boxes, screen(4, 1), Math.max(5, hexPx * 0.22) + 3)).toBe(true);
  });

  it("repaint a Settler's proposed city ring: the target tile and its six neighbours", () => {
    const boxes = hoverClipBoxes({ ...base, view: viewOf(unit('Settler', 0, 0)), selectedUnit: 7, path: [{ hex: { q: 3, r: 0 } }] as never });
    for (const [q, r] of [[3, 0], [4, 0], [2, 0], [3, 1], [3, -1], [4, -1], [2, 1]]) expect(covers(boxes, screen(q, r), HEX_SIZE * cam.zoom)).toBe(true);
    expect(hoverClipBoxes({ ...base, view: viewOf(unit('Settler', 0, 0)), selectedUnit: null, path: null })).toEqual([]);
  });
});
