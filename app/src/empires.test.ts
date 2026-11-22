import { expect, it } from 'vitest';
import { bearing, describeLocation, empireLocation, empireOfPlayer, empireTitle, placeName, unitsNearOurs } from './empires';

it('names a tile by the nearest known city, not by coordinates', () => {
  const v = view({ cities: [city(1, 0, 'Brasslock', 5, 5), city(2, 4, 'Cairnhold', 12, 5)] });
  expect(placeName(v, fromOffset(5, 5))).toBe('Brasslock');
  expect(placeName(v, fromOffset(7, 5))).toBe('2 tiles E of Brasslock');
  expect(placeName(v, fromOffset(11, 5))).toBe('1 tile W of Cairnhold');
  expect(placeName(v, fromOffset(30, 30))).toBeNull();
  expect(empireOfPlayer(v, 4)).toBe('Celts');
});
import { fromOffset } from './hex';
import type { PlayerView } from './types';

const player = (id: number, team: number, name: string, faction: string) => ({ id, team, name, faction, is_human: id === 0, alive: true, vp: 0, vp_sources: [], cities: 1, faction_goal_done: false });
const city = (id: number, owner: number, name: string, col: number, row: number) => ({ id, owner, name, pos: fromOffset(col, row), population: 1, hp: 100, max_hp: 100, strength: 10, is_capital: true, walls: false, visible: true, detail: null });
const unit = (id: number, owner: number, kind: string, col: number, row: number) => ({ id, owner, kind, pos: fromOffset(col, row), hp: 100, moves_left: 2, max_moves: 2, goto: null });

function view(parts: Record<string, unknown>): PlayerView {
  return {
    me: 0, team: 0,
    players: [player(0, 0, 'Ada', 'Byzantium'), player(4, 4, 'AI 4', 'Celts'), player(5, 5, 'AI 5', 'India')],
    cities: [], units: [],
    ...parts,
  } as unknown as PlayerView;
}

it('names an empire by its faction, adding the seat name when factions are shared', () => {
  const v = view({});
  // A seat default adds nothing: the seat number is on the crest beside the name.
  expect(empireTitle(v, 4)).toBe('Celts');
  // A name that restates the faction adds nothing either.
  v.players[1].name = 'Celts envoy';
  expect(empireTitle(v, 4)).toBe('Celts');
  // A distinct, player-given name does not lengthen ordinary sentences.
  expect(empireTitle(v, 0)).toBe('Byzantium');
  // Two teams on one faction are told apart by name.
  v.players[2] = { ...v.players[2], faction: 'Celts' };
  expect(empireTitle(v, 4)).toBe('Celts · Celts envoy');
  expect(empireTitle(v, 5)).toBe('Celts · AI 5');
  v.players[1].name = 'Celts';
  expect(empireTitle(v, 4)).toBe('Celts');
});

it('gives an 8-point compass bearing with north up', () => {
  const at = fromOffset(10, 10);
  expect(bearing(at, fromOffset(16, 10))).toBe('E');
  expect(bearing(at, fromOffset(4, 10))).toBe('W');
  expect(bearing(at, fromOffset(10, 2))).toBe('N');
  expect(bearing(at, fromOffset(10, 18))).toBe('S');
  expect(bearing(at, fromOffset(15, 5))).toBe('NE');
  expect(bearing(at, fromOffset(5, 15))).toBe('SW');
});

it('locates the nearest known city from our nearest city, else a unit in sight, else unknown', () => {
  const ours = [city(1, 0, 'Living Cairn', 5, 10), city(2, 0, 'Astra', 20, 10)];
  const v = view({ cities: [...ours, city(3, 4, 'Harmonia', 24, 10), city(4, 4, 'Far', 40, 10)], units: [unit(9, 4, 'Spearman', 6, 10)] });
  const loc = empireLocation(v, 4)!;
  // A known city beats a closer unit; it is measured from the closest of our cities.
  expect(loc).toMatchObject({ kind: 'city', name: 'Harmonia', tiles: 4, direction: 'E', from: 'Astra', at: fromOffset(24, 10) });
  expect(describeLocation(loc)).toBe('Nearest city Harmonia ~4 tiles E of Astra');

  const unitOnly = empireLocation(view({ cities: ours, units: [unit(9, 4, 'Spearman', 5, 7)] }), 4)!;
  expect(describeLocation(unitOnly)).toBe('City not seen yet; their Spearman ~3 tiles N of Living Cairn');

  expect(empireLocation(view({ cities: ours }), 5)).toBeNull();
  expect(describeLocation(null)).toBe('Location unknown: not met yet');
});

it('lists their units in sight near ours, nearest first', () => {
  const v = view({
    cities: [city(1, 0, 'Astra', 5, 10)],
    units: [unit(1, 0, 'Scout', 12, 10), unit(7, 4, 'Spearman', 13, 10), unit(8, 4, 'Scout', 8, 10), unit(9, 4, 'Settler', 30, 10)],
  });
  expect(unitsNearOurs(v, 4)).toEqual([
    { unit: 'Spearman', at: fromOffset(13, 10), tiles: 1, near: 'your Scout' },
    { unit: 'Scout', at: fromOffset(8, 10), tiles: 3, near: 'Astra' },
  ]);
});
