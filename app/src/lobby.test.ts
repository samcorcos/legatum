import { teamTarget } from './gameSetup';
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  addSlot,
  applyPreset,
  buildSetup,
  levelFields,
  levelOf,
  levelsFor,
  offeredLevels,
  PLAYER_LEVELS,
  readDifficultyPreference,
  TRAINING_DEV_KEY,
  trainingOffered,
  trainingSwitch,
  MAP_SIZES,
  LOBBY_DIFFICULTY_KEY,
  MAX_TURN_LIMIT,
  newLobby,
  notes,
  slotColorFactions,
  problems,
  removeSlot,
  rememberDifficulty,
  setDifficulty,
  setMapSize,
  setSlot,
  teams,
} from './lobby';

afterEach(() => vi.unstubAllGlobals());

function profile(): Map<string, string> {
  const data = new Map<string, string>();
  vi.stubGlobal('localStorage', { getItem: (key: string) => data.get(key) ?? null, setItem: (key: string, value: string) => data.set(key, value), removeItem: (key: string) => data.delete(key) });
  return data;
}

/** A profile with the developer switch for the Training level on. */
function researchProfile(): Map<string, string> {
  const data = profile();
  data.set(TRAINING_DEV_KEY, '1');
  return data;
}

describe('the levels a player is offered', () => {
  it('are exactly Easy, Normal and Hard', () => {
    profile();
    expect(trainingOffered()).toBe(false);
    expect(offeredLevels()).toEqual(['Easy', 'Normal', 'Hard']);
    expect(levelsFor(false)).toEqual(['Easy', 'Normal', 'Hard']);
    expect([...PLAYER_LEVELS]).toEqual(['Easy', 'Normal', 'Hard']);
    vi.stubGlobal('location', { search: '?training=1' });
    vi.stubGlobal('localStorage', undefined);
    expect(trainingOffered()).toBe(true); // Vitest runs as a development build.
  });

  it('a production build never offers Training, whatever the URL or the browser remembers', () => {
    const store = { getItem: () => '1', setItem: () => undefined, removeItem: () => undefined };
    expect(trainingSwitch(false, '?training=1', store)).toBe(false);
    expect(trainingSwitch(false, '', store)).toBe(false);
    expect(levelsFor(trainingSwitch(false, '?training=1', store))).toEqual(['Easy', 'Normal', 'Hard']);
  });

  it('a development build offers Training only when asked, and remembers the answer', () => {
    const data = profile();
    const store = globalThis.localStorage;
    expect(trainingSwitch(true, '', store)).toBe(false);
    expect(trainingSwitch(true, '?training=1', store)).toBe(true);
    expect(data.get(TRAINING_DEV_KEY)).toBe('1');
    expect(trainingSwitch(true, '', store)).toBe(true);
    expect(levelsFor(true)).toEqual(['Easy', 'Normal', 'Hard', 'Training']);
    expect(trainingSwitch(true, '?training=0', store)).toBe(false);
    expect(trainingSwitch(true, '', store)).toBe(false);
    expect(trainingSwitch(true, '?training=1', null)).toBe(true);
  });

  it('a stored Training or HardSearch preference reads as Hard, ignoring old seat overrides', () => {
    for (const old of ['Training', 'HardSearch']) {
      const data = profile();
      data.set(LOBBY_DIFFICULTY_KEY, JSON.stringify({ level: old, seats: { 'solo:2': old, 'solo:3': 'Easy' } }));
      expect(readDifficultyPreference()).toEqual({ level: 'Hard' });
      const lobby = newLobby('solo', '9');
      expect(levelOf(lobby)).toBe('Hard');
      expect(lobby.slots.map((s) => s.kind === 'ai' ? levelOf(s) : null)).toEqual([null, 'Hard', 'Hard', 'Hard', 'Hard', 'Hard']);
      expect(buildSetup(lobby).players.some((p) => p.search)).toBe(false);
      // Opening the lobby rewrote the preference without the level that is not offered.
      expect(data.get(LOBBY_DIFFICULTY_KEY)).not.toMatch(/Training|HardSearch/);
    }
  });

  it('a lobby that already says Training plays plain Hard', () => {
    profile();
    const lobby = newLobby('solo', '9');
    const stale = { ...lobby, search: true, slots: lobby.slots.map((s) => s.kind === 'ai' ? { ...s, difficulty: 'Hard' as const, search: true } : s) };
    expect(levelOf(stale)).toBe('Hard');
    expect(stale.slots.slice(1).every((s) => levelOf(s) === 'Hard')).toBe(true);
    expect(buildSetup(stale).players.some((p) => p.search)).toBe(false);
    expect(buildSetup(setDifficulty(lobby, 'Training')).players.some((p) => p.search)).toBe(false);
  });
});

describe('lobby', () => {
  it('defaults every AI seat to Hard and writes Hard into setup', () => {
    profile();
    const lobby = newLobby('solo', '42');
    expect(levelOf(lobby)).toBe('Hard');
    expect(lobby.slots.slice(1).every((seat) => levelOf(seat) === 'Hard')).toBe(true);
    expect(buildSetup(lobby).players.slice(1).every((player) => player.difficulty === 'Hard')).toBe(true);
    const converted = setSlot(newLobby('host', '42'), 1, { kind: 'ai' });
    expect(converted.slots[1].difficulty).toBe('Hard');
  });

  it('keeps the game-wide choice and discards old individual overrides across a version bump', () => {
    const data = researchProfile();
    const chosen = setSlot(setDifficulty(newLobby('solo', '1'), 'Easy'), 2, levelFields('Training'));
    rememberDifficulty(chosen);
    data.set('legatum.autosave.v2', '{}'); // A release can change its save format and key.
    const reopened = newLobby('solo', '2');
    expect(levelOf(reopened)).toBe('Easy');
    expect(levelOf(reopened.slots[1])).toBe('Easy');
    expect(levelOf(reopened.slots[2])).toBe('Easy');
    expect(LOBBY_DIFFICULTY_KEY).not.toMatch(/v\d/);
  });

  it('does not restore a solo seat override when another lobby mode opens', () => {
    profile();
    rememberDifficulty(setSlot(newLobby('solo', '1'), 1, levelFields('Normal')));
    newLobby('host', '2');
    expect(levelOf(newLobby('solo', '3').slots[1])).toBe('Hard');
  });

  it('reports when a chosen difficulty cannot be persisted', () => {
    vi.stubGlobal('localStorage', { getItem: () => null, setItem: () => { throw new Error('Storage blocked'); } });
    expect(rememberDifficulty(setDifficulty(newLobby('solo', '5'), 'Easy'))).toBe(false);
  });
  it('plays every game without a limit by default; preserves explicit limits and VP targets', () => {
    const l = newLobby('solo', '42');
    expect(l.turnLimit).toBe(null);
    for (const turnLimit of [null, 1, 275, MAX_TURN_LIMIT]) {
      const lobby = { ...l, turnLimit };
      expect(problems(lobby)).toEqual([]);
      expect(buildSetup(lobby)).toMatchObject({ turn_limit: turnLimit });
      expect(teams(lobby).map((t) => t.target)).toEqual(teams(l).map((t) => t.target));
    }
  });

  it('rejects empty, fractional, negative, non-finite and overflowing enabled turn limits', () => {
    for (const turnLimit of [0, -1, 1.5, Infinity, NaN, MAX_TURN_LIMIT + 1]) {
      expect(problems({ ...newLobby('solo', '42'), turnLimit }).join()).toMatch(/Turn limit/);
    }
  });
  it('single player defaults to a free-for-all on Standard: you + 5 AIs, all on their own teams, one solo target each', () => {
    const l = newLobby('solo', '42');
    expect(l.preset).toBe('ffa');
    expect(l.mapSize).toBe('Standard');
    expect(l.slots.map((s) => s.kind)).toEqual(['you', 'ai', 'ai', 'ai', 'ai', 'ai']);
    const s = buildSetup(l);
    expect(s.players).toHaveLength(6);
    expect(s.players.filter((p) => p.is_human)).toHaveLength(1);
    expect(new Set(s.players.map((p) => p.team)).size).toBe(6);
    expect(teams(l).every((t) => t.target === teamTarget(1))).toBe(true);
    expect([s.width, s.height, s.turn_limit]).toEqual([48, 32, null]);
    expect(problems(l)).toEqual([]);
  });

  it('online hosting defaults to co-op vs AI: the humans share a team with a two-empire target', () => {
    const l = newLobby('host', '42');
    expect(l.preset).toBe('coop');
    const t = teams(l);
    expect(t[0].slots).toEqual([0, 1]);
    expect(t[0].target).toBe(teamTarget(2));
    expect(t.slice(1).every((x) => x.target === teamTarget(1))).toBe(true);
    expect(problems(l)).toEqual([]);
  });

  it('teams preset: four teams of two; a team of three needs three solo targets', () => {
    let l = applyPreset(newLobby('solo', '1'), 'teams');
    expect(teams(l).map((t) => t.slots.length)).toEqual([2, 2, 2, 2]);
    expect(teams(l)[0].slots).toEqual([0, 4]);
    l = setSlot(l, 1, { team: 0 });
    expect(l.preset).toBe('custom');
    expect(teams(l)[0].target).toBe(teamTarget(3));
  });

  it('puts humans first in player order and gives each seat its faction\'s colour', () => {
    let l = newLobby('host', '9');
    l = setSlot(l, 1, { kind: 'ai' });
    l = setSlot(l, 5, { kind: 'human' });
    expect(l.slots[5].name).toBe('Player 6');
    l = setSlot(setSlot(l, 0, { faction: 'Sparta' }), 5, { faction: 'Athens' });
    const colored = slotColorFactions(l);
    expect(colored[0]).toBe('Sparta');
    expect(colored[5]).toBe('Athens');
    const s = buildSetup(l);
    expect(s.players.slice(0, 2).map((p) => p.is_human)).toEqual([true, true]);
    expect(s.players.slice(2).every((p) => !p.is_human)).toBe(true);
  });

  it('closed seats are left out; map size notes and small-map setup', () => {
    let l = newLobby('solo', '5');
    for (let i = 5; i >= 4; i--) l = setSlot(l, i, { kind: 'closed' });
    expect(notes(l)[0]).toMatch(/Small is recommended/);
    l = { ...l, mapSize: 'Small' };
    expect(notes(l)).toEqual([]);
    const s = buildSetup(l);
    expect(s.players).toHaveLength(4);
    expect([s.width, s.height, s.turn_limit]).toEqual([40, 26, null]);
  });

  it('blocks games with no rivals, too few empires, or a bad seed', () => {
    let l = newLobby('solo', '5');
    for (let i = 1; i < 8; i++) l = setSlot(l, i, { team: 0 });
    expect(problems(l).join()).toMatch(/one team/);
    l = newLobby('solo', 'abc');
    expect(problems(l).join()).toMatch(/seed/);
    l = newLobby('solo', '5');
    for (let i = 7; i >= 2; i--) l = removeSlot(l, i);
    expect(l.slots).toHaveLength(2);
    expect(removeSlot(l, 1).slots).toHaveLength(2);
    l = setSlot(l, 1, { kind: 'closed' });
    expect(problems(l).join()).toMatch(/two empires/);
  });

  it('network host allows 1-8 humans, including the direct Host Game path', () => {
    let l = newLobby('host', '5');
    expect(problems(l)).toEqual([]);
    l = setSlot(l, 2, { kind: 'human' });
    expect(problems(l)).toEqual([]);
    l = setSlot(setSlot(l, 1, { kind: 'ai' }), 2, { kind: 'ai' });
    expect(problems(l)).toEqual([]);
    for (let i = 1; i < 8; i++) l = setSlot(l, i, { kind: 'human' });
    expect(problems(l)).toEqual([]);
    expect(buildSetup(l).players.filter((p) => p.is_human)).toHaveLength(8);
  });

  it('removing a network row closes its stable slot without shifting another player', () => {
    let l = setSlot(newLobby('host', '5'), 1, { name: 'Alice' });
    l = setSlot(setSlot(l, 2, { kind: 'human' }), 2, { name: 'Bob' });
    const next = removeSlot(l, 1);
    expect(next.slots).toHaveLength(l.slots.length);
    expect(next.slots[1].kind).toBe('closed');
    expect(next.slots[2]).toEqual(l.slots[2]);
    expect(buildSetup(next).players.slice(0, 2).map((p) => p.name)).toEqual(['Player 1', 'Bob']);
  });

  it('difficulty: default applies to all AIs; only non-Normal is written to the setup', () => {
    let l = setDifficulty(newLobby('solo', '5'), 'Hard');
    l = setSlot(l, 3, { difficulty: 'Normal' });
    const s = buildSetup(l);
    expect(s.players[0].difficulty).toBeUndefined();
    expect(s.players[1].difficulty).toBe('Hard');
    expect(s.players[3].difficulty).toBeUndefined();
  });

  it('Training (developer switch on): Hard with the search flag, applied to AI seats, written to the setup only for it', () => {
    researchProfile();
    let l = setDifficulty(newLobby('solo', '5'), 'Training');
    expect(levelOf(l)).toBe('Training');
    expect(l.slots.filter((s) => s.kind === 'ai').every((s) => levelOf(s) === 'Training')).toBe(true);
    const s = buildSetup(l);
    expect(s.players[0]).not.toHaveProperty('search');
    expect(s.players[1]).toMatchObject({ difficulty: 'Hard', search: true });
    // A seat set back to plain Hard, Easy or Normal drops the flag.
    l = setSlot(l, 2, levelFields('Hard'));
    l = setSlot(l, 3, levelFields('Easy'));
    const b = buildSetup(l);
    expect(b.players[2]).toMatchObject({ difficulty: 'Hard' });
    expect(b.players[2]).not.toHaveProperty('search');
    expect(b.players[3]).toMatchObject({ difficulty: 'Easy' });
    expect(b.players[3]).not.toHaveProperty('search');
    // A seat added later takes the default level, and the lobby default survives a preset change.
    expect(levelOf(addSlot(removeSlot(l, 5)).slots[5])).toBe('Training');
    expect(levelOf(applyPreset(l, 'teams').slots[5])).toBe('Training');
    // Plain Hard never writes the flag, and the flag alone (without Hard) is ignored.
    expect(buildSetup(setDifficulty(newLobby('solo', '5'), 'Hard')).players[1]).not.toHaveProperty('search');
    expect(levelOf({ difficulty: 'Normal', search: true })).toBe('Normal');
  });

  it('random factions come from the seed (same seed, same empires)', () => {
    const a = buildSetup(newLobby('solo', '123')).players.map((p) => p.faction);
    const b = buildSetup(newLobby('solo', '123')).players.map((p) => p.faction);
    expect(a).toEqual(b);
    const l = addSlot(removeSlot(newLobby('solo', '123'), 5));
    expect(l.slots).toHaveLength(6);
    expect(l.slots[5].kind).toBe('ai');
    expect(new Set(l.slots.map((s) => s.team)).size).toBe(6);
  });

  describe('setMapSize snaps the empire count into the size range', () => {
    const count = (l: ReturnType<typeof newLobby>) => l.slots.filter((x) => x.kind !== 'closed').length;
    /** A solo lobby grown to all eight seats (the default is six), for the tests that need a full table. */
    const full = () => addSlot(addSlot(newLobby('solo', '1')));

    it('Small trims the default 6 to 4 from the bottom, keeping the human and the first AIs', () => {
      const l = setMapSize(newLobby('solo', '1'), 'Small');
      expect(l.mapSize).toBe('Small');
      expect(l.slots.map((s) => s.name)).toEqual(['Player 1', 'AI 2', 'AI 3', 'AI 4']);
      expect(notes(l)).toEqual([]);
      expect(problems(l)).toEqual([]);
    });

    it('Small to Standard raises 4 to the Standard default, 6 (not its minimum, 5), with default AI seats', () => {
      const small = setMapSize(newLobby('solo', '1'), 'Small');
      expect(count(small)).toBe(4);
      const l = setMapSize(small, 'Standard');
      expect(count(l)).toBe(6);
      expect(l.mapSize).toBe('Standard');
      expect(l.slots.slice(4).every((s) => s.kind === 'ai' && s.faction === 'Random')).toBe(true);
      expect(notes(l)).toEqual([]);
      const three = setMapSize(removeSlot(small, 3), 'Standard');
      expect(count(three)).toBe(6);
    });

    it('untouched AI seats reset to each size default, even when the old count fits', () => {
      expect(count(setMapSize(newLobby('solo', '1'), 'Standard'))).toBe(6);
      expect(count(setMapSize(full(), 'Standard'))).toBe(MAP_SIZES.Standard.default);
      const five = removeSlot(newLobby('solo', '1'), 5);
      expect(count(setMapSize(five, 'Standard'))).toBe(MAP_SIZES.Standard.default);
      expect(count(setMapSize(five, 'Small'))).toBe(4);
    });

    it('Large to Standard returns to six; Small to Large raises to eight', () => {
      const large = setMapSize(setMapSize(newLobby('solo', '1'), 'Small'), 'Large');
      expect(count(large)).toBe(8);
      expect(large.mapSize).toBe('Large');
      expect(count(setMapSize(large, 'Standard'))).toBe(MAP_SIZES.Standard.default);
    });

    it('keeps an in-range count after a name or faction choice', () => {
      const named = setSlot(full(), 1, { name: 'Boudica' });
      expect(count(setMapSize(named, 'Standard'))).toBe(8);
      const chosen = setSlot(removeSlot(newLobby('solo', '1'), 5), 2, { faction: 'Celts' });
      expect(count(setMapSize(chosen, 'Standard'))).toBe(5);
    });

    it('each map size has a default empire count inside its own range', () => {
      expect(Object.values(MAP_SIZES).map((s) => s.default)).toEqual([4, 6, 8]);
      for (const s of Object.values(MAP_SIZES)) expect(s.default >= s.min && s.default <= s.max).toBe(true);
    });

    it('free-for-all starts at the map size\'s default; Teams and Co-op keep all eight seats', () => {
      const ffa = (mapSize: 'Small' | 'Standard' | 'Large') => applyPreset({ ...newLobby('solo', '1'), mapSize }, 'ffa');
      expect(ffa('Small').slots).toHaveLength(4);
      expect(ffa('Standard').slots).toHaveLength(6);
      expect(ffa('Large').slots).toHaveLength(8);
      expect(applyPreset(newLobby('solo', '1'), 'teams').slots).toHaveLength(8);
      expect(applyPreset(newLobby('solo', '1'), 'coop').slots).toHaveLength(8);
      const coop = newLobby('host', '1');
      expect(coop.slots.map((s) => s.kind)).toEqual(['you', 'human', 'ai', 'ai', 'ai', 'ai', 'ai', 'ai']);
    });

    it('the add-seat and remove controls still reach anywhere from 2 to 8', () => {
      let l = newLobby('solo', '1');
      expect(count(l)).toBe(6);
      l = addSlot(addSlot(addSlot(l)));
      expect(count(l)).toBe(8);
      expect(count(addSlot(l))).toBe(8);
      l = removeSlot(l, 7);
      expect(count(l)).toBe(7);
      for (let i = 0; i < 10; i++) l = removeSlot(l, l.slots.length - 1);
      expect(count(l)).toBe(2);
      expect(l.slots[0].kind).toBe('you');
    });

    it('keeps customised AI seats (named or with a chosen faction) and the warning shows', () => {
      let l = full();
      for (let i = 1; i < 8; i++) l = setSlot(l, i, i % 2 ? { faction: 'Celts' } : { name: 'Rival' });
      const s = setMapSize(l, 'Small');
      expect(count(s)).toBe(8);
      expect(notes(s)).toHaveLength(1);
      // one default seat among the customised ones is the only one trimmed
      let m = full();
      for (let i = 1; i < 8; i++) if (i !== 5) m = setSlot(m, i, { faction: 'Celts' });
      expect(count(setMapSize(m, 'Small'))).toBe(7);
    });

    it('never removes a human seat', () => {
      let l = newLobby('host', '1');
      l = setSlot(l, 7, { kind: 'human' });
      l = setSlot(l, 6, { kind: 'human' });
      const s = setMapSize(l, 'Small');
      expect(s.slots.filter((x) => x.kind === 'human' || x.kind === 'you')).toHaveLength(4);
      expect(count(s)).toBe(4);
      expect(s.slots.slice(0, 2).map((x) => x.kind)).toEqual(['you', 'human']);
    });

    it('co-op keeps the humans on one team against at least one other team', () => {
      const s = setMapSize(newLobby('host', '1'), 'Small');
      expect(count(s)).toBe(4);
      expect(teams(s)[0].slots).toEqual([0, 1]);
      expect(problems(s)).toEqual([]);
    });

    it('teams preset trims to four seats and stays valid', () => {
      const s = setMapSize(applyPreset(newLobby('solo', '1'), 'teams'), 'Small');
      expect(count(s)).toBe(4);
      expect(problems(s)).toEqual([]);
    });

    it('does not trim away the last seat of a second team', () => {
      let l = full();
      for (let i = 1; i < 8; i++) l = setSlot(l, i, { team: i < 4 ? 0 : 1 });
      const s = setMapSize(l, 'Small');
      expect(count(s)).toBe(4);
      expect(teams(s)).toHaveLength(2);
      expect(problems(s)).toEqual([]);
    });

    it('network rooms close trimmed AI seats without shifting ids, and reopen them when raising', () => {
      let l = newLobby('host', '1');
      l = setSlot(l, 2, { kind: 'ai' });
      l = applyPreset({ ...l, mapSize: 'Large' }, 'ffa');
      expect(l.slots).toHaveLength(8);
      l = setSlot(l, 1, { kind: 'human' });
      const s = setMapSize(l, 'Small');
      expect(s.slots).toHaveLength(8);
      expect(s.slots.slice(4).every((x) => x.kind === 'closed')).toBe(true);
      expect(s.slots[1].kind).toBe('human');
      const back = setMapSize(s, 'Standard');
      expect(back.slots).toHaveLength(8);
      expect(count(back)).toBe(6);
      expect(back.slots[4].kind).toBe('ai');
      expect(back.slots[5].kind).toBe('ai');
    });

    it('the player may still go outside the range afterwards', () => {
      const l = addSlot(setMapSize(newLobby('solo', '1'), 'Small'));
      expect(l.mapSize).toBe('Small');
      expect(notes(l)).toHaveLength(1);
    });
  });
});
