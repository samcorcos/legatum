import { expect, it } from 'vitest';
import { pointerIntent, previewDestination } from './intent';
import type { PlayerView, UnitView } from '../types';

const unit = (id: number, owner: number, q: number, r: number, military = true) => ({ id, owner, pos: { q, r }, military, kind: military ? 'Spearman' : 'Settler' }) as unknown as UnitView;
const view = {
  me: 0,
  team: 0,
  players: [{ id: 0, team: 0 }, { id: 1, team: 1 }],
  units: [unit(1, 0, 0, 0), unit(2, 0, 2, 0, false), unit(3, 1, 4, 0)],
  cities: [{ id: 10, owner: 0, pos: { q: 0, r: 3 } }, { id: 11, owner: 1, pos: { q: 5, r: 5 } }],
} as unknown as PlayerView;
const spearman = view.units[0];

it('previews a move only where the command would move or attack', () => {
  expect(pointerIntent(view, spearman, { q: 1, r: 1 }, 'normal')).toBe('move');
  expect(pointerIntent(view, spearman, { q: 4, r: 0 }, 'normal')).toBe('attack');
  expect(pointerIntent(view, spearman, { q: 5, r: 5 }, 'normal')).toBe('attack');
});

it("shows a select affordance over the player's own units and cities, not a path", () => {
  // The user's report: with a unit selected, hovering another own unit drew a move path.
  expect(pointerIntent(view, spearman, { q: 2, r: 0 }, 'normal')).toBe('select');
  expect(pointerIntent(view, spearman, { q: 0, r: 3 }, 'normal')).toBe('select');
  expect(pointerIntent(view, null, { q: 2, r: 0 }, 'normal')).toBe('select');
  expect(pointerIntent(view, null, { q: 1, r: 1 }, 'normal')).toBe('none');
});

it('previews nothing on the selected unit itself, while ranged targeting, or for a civilian over an enemy', () => {
  expect(pointerIntent(view, spearman, spearman.pos, 'normal')).toBe('none');
  expect(pointerIntent(view, spearman, { q: 1, r: 1 }, 'ranged')).toBe('none');
  expect(pointerIntent(view, view.units[1], { q: 4, r: 0 }, 'normal')).toBe('none');
});

it('offers selection for another own unit sharing the selected tile', () => {
  const stacked = { ...view, units: [spearman, { ...view.units[1], pos: spearman.pos }] };
  expect(pointerIntent(stacked, spearman, spearman.pos, 'normal')).toBe('select');
});

it('previews destinations over own cities and escorts during explicit move targeting', () => {
  expect(pointerIntent(view, spearman, { q: 0, r: 3 }, 'move')).toBe('move');
  expect(pointerIntent(view, spearman, { q: 2, r: 0 }, 'move')).toBe('move');
  expect(pointerIntent(view, spearman, spearman.pos, 'move')).toBe('none');
});

it('hides a queued route while hovering a select target and restores it away from units', async () => {
  const { previewDestination } = await import('./intent');
  const pending = { q: 7, r: 4 };
  expect(previewDestination('select', view.units[1].pos, null, pending)).toBeNull();
  expect(previewDestination('none', null, null, pending)).toEqual(pending);
  expect(previewDestination('move', { q: 1, r: 1 }, null, pending)).toEqual({ q: 1, r: 1 });
});

it('keeps a drag path onto an own unit for escort stacking', () => {
  const target = view.units[1].pos;
  expect(previewDestination(pointerIntent(view, spearman, target, 'normal'), target, target, null)).toEqual(target);
});
it('keeps an explicit settlement preview when the mouse passes over an own unit', () => {
  const target = { q: 7, r: 4 };
  expect(previewDestination('select', spearman.pos, target, null)).toEqual(target);
});
