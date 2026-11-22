// End-of-game states for Victory and Defeat screens, including the longest eight-empire standings.
// These are injected visual fixtures, not gameplay examples.
import type { Faction } from '../types';
import { coreFixture } from './coreFixture';
import { createState, makeStandardSetup } from './fixtureEngine';
import type { MState } from './fixtureState';

export type GameOverKind = 'victory' | 'team-victory' | 'rival-victory' | 'defeat';

const AI: Faction[] = ['Switzerland', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens'];

function award(s: MState, p: number, goal: boolean, eras = 6) {
  s.players[p].factionGoalDone = goal;
  s.players[p].eras = Object.fromEntries(Array.from({ length: eras }, (_, i) => [i + 2, { entered: 1, scored: { turn: 40 + i, slot: 0 } }]));
}

function heldWork(s: MState, p: number) {
  s.players[p].greatWorkCompleted = 'RelayUplink';
  s.players[p].greatWorkHolder = p;
  s.owner[0] = p;
  s.landmark = Array(s.terrain.length).fill(null);
  s.landmark[0] = { building: 'RelayUplink', city: 0, of: p };
}

function gameOverState(kind: GameOverKind): MState {
  const setup = makeStandardSetup('700077', 'India', 'Byzantium', 'Ada', 'Bo');
  setup.players = setup.players.map((p, i) => (i >= 2 ? { ...p, faction: AI[i - 2] } : p));
  if (kind !== 'team-victory') setup.players[1] = { ...setup.players[1], is_human: false, team: 7 };
  const s = createState(setup);
  s.turn = 164;
  for (const p of s.players) { p.gold = 300; p.explored.fill(1); }
  const ids = Object.fromEntries(s.players.map((p) => [p.faction, p.id]));
  if (kind === 'team-victory') {
    award(s, 0, true);
    award(s, 1, true);
    award(s, ids.Celts, false, 3);
    award(s, ids.Sparta, true, 1);
    heldWork(s, 0);
    s.finished = true; s.winner = 0;
  } else if (kind === 'victory') {
    award(s, 0, true);
    award(s, ids.Celts, true, 3);
    award(s, ids.Switzerland, false, 2);
    s.finished = true; s.winner = 0;
  } else if (kind === 'rival-victory') {
    award(s, ids.Celts, true);
    award(s, 0, false, 3);
    s.finished = true; s.winner = s.players[ids.Celts].team;
  } else {
    award(s, ids.Celts, true, 3);
    award(s, ids.Switzerland, false, 2);
    award(s, ids.Germany, true, 1);
    s.players[0].alive = false;
    s.units = s.units.filter((u) => u.owner !== 0);
    s.cities = s.cities.filter((c) => c.owner !== 0);
    // The fallen empire's territory goes with its cities: core refuses a save whose tiles name a missing city.
    const kept = new Set(s.cities.map((c) => c.id));
    s.tileCity = s.tileCity.map((c) => (c >= 0 && !kept.has(c) ? -1 : c));
    s.owner = s.owner.map((o, i) => (s.tileCity[i] < 0 && o === 0 ? -1 : o));
  }
  return s;
}

export function gameOverCoreSave(kind: GameOverKind): string {
  return coreFixture(gameOverState(kind));
}
