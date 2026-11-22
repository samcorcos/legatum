import { expect, it } from 'vitest';
import type { PlayerView, UnitView } from './types';
import { BARBARIAN, campAt, CAMP_CLEAR_GOLD, ownerName } from './barbarians';
import { atWar } from './diplomacy';
import { enemyAt } from './commands';
import { pointerIntent } from './map/intent';
import { formatEvent, type ReportContext } from './format';
import { newLobby, buildSetup } from './lobby';
import { configFromLobby } from './staging';
import { playerColor, FACTION_PALETTE } from './theme';

const unit = (id: number, owner: number, q: number, r: number, military = true) =>
  ({ id, owner, pos: { q, r }, military, kind: military ? 'Soldier' : 'Settler' }) as unknown as UnitView;
// Two empires at Peace, a barbarian Soldier and a remembered camp.
const view = {
  me: 0,
  team: 0,
  players: [{ id: 0, team: 0, name: 'Ana', faction: 'Byzantium' }, { id: 1, team: 1, name: 'Ben', faction: 'India' }],
  diplomacy: { legacy: false, relations: [{ a: 0, b: 1, state: 'Peace' }] },
  units: [unit(1, 0, 0, 0), unit(2, 1, 2, 0), unit(3, BARBARIAN, 4, 0)],
  cities: [],
  barbarian_camps: [{ pos: { q: 6, r: 0 }, visible: false }],
} as unknown as PlayerView;

it('barbarians are always at war, with no Declare War step, while a Peace neighbour is not', () => {
  expect(atWar(view, BARBARIAN)).toBe(true);
  expect(atWar(view, 1)).toBe(false);
  expect(enemyAt(view, { q: 4, r: 0 })).toEqual({ kind: 'unit', owner: BARBARIAN });
  expect(enemyAt(view, { q: 2, r: 0 })).toBeNull();
  expect(pointerIntent(view, view.units[0], { q: 4, r: 0 }, 'normal')).toBe('attack');
});

it('names and colors barbarians apart from every seat', () => {
  expect(ownerName(view, BARBARIAN)).toBe('Barbarians');
  expect(ownerName(view, 1)).toBe('India');
  expect(ownerName(view, 0)).toBe('You');
  expect(Object.values(FACTION_PALETTE).map((p) => p.color)).not.toContain(playerColor(BARBARIAN));
  expect(campAt(view, { q: 6, r: 0 })).toEqual({ pos: { q: 6, r: 0 }, visible: false });
  expect(campAt(view, { q: 5, r: 0 })).toBeNull();
});

it('pays the fixed Standard clearing reward', () => {
  expect(CAMP_CLEAR_GOLD).toBe(50);
});

const ctx: ReportContext = {
  me: 0,
  team: [0],
  playerName: (id) => (id === null ? 'nobody' : id === BARBARIAN ? 'Barbarians' : ['Ana', 'Ben'][id] ?? `E${id}`),
  unitKind: (id) => (id === 7 ? 'Soldier' : null),
  cityName: (id) => (id === 1 ? 'Tithe Station' : 'a city'),
  cityPos: (id) => (id === 1 ? { q: 2, r: 2 } : null),
};

it('reports camps, scouts, raids and clearing in the turn report', () => {
  expect(formatEvent({ type: 'BarbarianCampDiscovered', camp: 0, team: 0, at: { q: 6, r: 0 } }, ctx)).toMatchObject({
    category: 'discoveries', text: expect.stringContaining('Barbarian camp sighted'), details: [expect.stringContaining('50 Gold')],
  });
  expect(formatEvent({ type: 'BarbarianScoutSpotted', unit: 9, city: 1, owner: 0, at: { q: 4, r: 2 } }, ctx)).toMatchObject({
    tone: 'bad', weight: 2, text: expect.stringContaining('Kill it before it reaches its camp'),
  });
  expect(formatEvent({ type: 'BarbarianRaidLaunched', camp: 0, city: 1, owner: 0, direction: 'north-east' }, ctx)).toMatchObject({
    at: { q: 2, r: 2 }, text: 'Barbarian raiders are gathering to raid Tithe Station from a camp to the north-east',
  });
  expect(formatEvent({ type: 'BarbarianCampCleared', camp: 0, at: { q: 6, r: 0 }, by: 0, unit: 7, gold: 50 }, ctx).text)
    .toBe('Your Spearman cleared a barbarian camp at this tile: +50 Gold');
  expect(formatEvent({ type: 'BarbarianCampCleared', camp: 0, at: { q: 6, r: 0 }, by: 1, unit: null, gold: 0 }, ctx).text)
    .toBe("Ben's borders dispersed a barbarian camp at this tile");
  expect(formatEvent({ type: 'SettlerCaptured', unit: 4, from: 0, to: BARBARIAN, at: { q: 1, r: 1 } }, ctx)).toMatchObject({
    text: 'Barbarians captured your Settler at this tile', tone: 'bad',
  });
  expect(formatEvent({ type: 'SettlerCaptured', unit: 4, from: BARBARIAN, to: 0, at: { q: 1, r: 1 } }, ctx).text)
    .toBe('You freed a captive Settler from the barbarians at this tile');
  const battle = formatEvent({
    type: 'Battle', attacker: 11, attacker_owner: BARBARIAN, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Soldier' }, at: { q: 1, r: 0 },
    attacker_strength: { total: 20, parts: [['Soldier', 20]] }, defender_strength: { total: 20, parts: [['Soldier', 20]] },
    damage_to_attacker: 30, damage_to_defender: 30, attacker_roll: 100, defender_roll: 100, attacker_hp_after: 70, defender_hp_after: 70, ranged: false, meeting: false,
  }, { ...ctx, unitKind: (id) => (id === 11 ? 'Soldier' : id === 7 ? 'Soldier' : null) });
  expect(battle.text.startsWith('Barbarian Spearman attacked your Spearman')).toBe(true);
  expect(battle.tone).toBe('bad');
});

it('new games have barbarians on; the setting reaches the setup and stays out of default rooms', () => {
  const lobby = newLobby('solo', '42');
  expect(lobby.barbarians).toBe(true);
  expect(buildSetup(lobby).barbarians).toBe(true);
  expect(buildSetup({ ...lobby, barbarians: false }).barbarians).toBe(false);
  expect(buildSetup({ ...lobby, barbarians: undefined }).barbarians).toBe(true);
  expect('barbarians' in configFromLobby(newLobby('host', '42'))).toBe(false);
  expect(configFromLobby({ ...newLobby('host', '42'), barbarians: false }).barbarians).toBe(false);
});
