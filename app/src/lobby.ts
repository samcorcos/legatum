// The game lobby (staging room) as plain data: seats, presets and game settings, and the
// GameSetup they produce. The Lobby screen renders this; the preference boundary is below.
import { FACTIONS } from './data';
import { MAX_PLAYERS, factionChoiceIndex, teamTarget } from './gameSetup';
import { colorFactions } from './theme';
import type { AiDifficulty, Faction, GameSetup, PlayerSetup } from './types';

/** solo: you + AIs · hotseat: several humans on this computer · host: humans join over the network. */
export type LobbyMode = 'solo' | 'hotseat' | 'host';
/** `you` is always the first seat: the person at this computer (the host in network games). */
export type SlotKind = 'you' | 'human' | 'ai' | 'closed';
export type FactionChoice = Faction | 'Random';
export type PresetId = 'ffa' | 'teams' | 'coop' | 'custom';
export type MapSizeId = 'Small' | 'Standard' | 'Large';
/** API-created rooms can use dimensions that do not match a preset. */
export type MapSize = MapSizeId | { width: number; height: number };

export interface Slot {
  kind: SlotKind;
  name: string;
  faction: FactionChoice;
  /** 0-based; shown as "Team n+1". Same team = permanent alliance, shared vision and victory. */
  team: number;
  difficulty: AiDifficulty;
  /** Training: a Hard AI whose battles are searched (the core's Hard with `search`). */
  search?: boolean;
}

export interface Lobby {
  mode: LobbyMode;
  preset: PresetId;
  slots: Slot[];
  mapSize: MapSize;
  turnLimit: number | null;
  /** Barbarian camps (core `GameSetup::barbarians`). On for new games; missing reads as on. */
  barbarians?: boolean;
  /** The AI-only teams start in Alliances of two (core `GameSetup::ai_alliances`). Co-op turns it on; missing reads as off. */
  aiAlliances?: boolean;
  seed: string;
  /** Default for new AI seats; setting it applies to every AI seat. */
  difficulty: AiDifficulty;
  search?: boolean;
  /** Coach questions: Hard plus "why did you do that?" for the human seat. App-side only; never in the game setup. */
  training?: boolean;
}

/** The largest turn limit the core accepts (a u32 less one). */
export const MAX_TURN_LIMIT = 4_294_967_294;
export const validTurnLimit = (limit: number | null): boolean => limit === null || (Number.isInteger(limit) && limit >= 1 && limit <= MAX_TURN_LIMIT);

export const MIN_SEATS = 2;
/** A level a seat can be recorded with. Players are offered Easy, Normal and Hard only. */
export type AiLevel = AiDifficulty | 'Training';
/** The levels every build offers. All three plan from the seat's own view of the map. */
export const PLAYER_LEVELS: readonly AiLevel[] = ['Easy', 'Normal', 'Hard'];

/**
 * Training (the core's Hard with `search`) is a research tool: its battle study reads the true
 * state, including units the seat cannot see, so it is not a level players are offered. A
 * development build offers it with `?training=1` in the URL (remembered in this browser under
 * `legatum.dev.training`; `?training=0` forgets it). A production build never does.
 */
export const TRAINING_DEV_KEY = 'legatum.dev.training';
export function trainingSwitch(dev: boolean, search: string, storage: Pick<Storage, 'getItem' | 'setItem' | 'removeItem'> | null | undefined): boolean {
  if (!dev) return false;
  const flag = new URLSearchParams(search).get('training');
  try {
    if (flag === '1') storage?.setItem(TRAINING_DEV_KEY, '1');
    else if (flag === '0') storage?.removeItem(TRAINING_DEV_KEY);
    if (flag === '0' || flag === '1') return flag === '1';
    return storage?.getItem(TRAINING_DEV_KEY) === '1';
  } catch { return flag === '1'; }
}
const storage = () => { try { return globalThis.localStorage; } catch { return null; } };
export const trainingOffered = (): boolean => trainingSwitch(import.meta.env?.DEV === true, globalThis.location?.search ?? '', storage());

/** What the AI-level controls offer: Easy, Normal and Hard, plus Training behind the developer switch. */
export function levelsFor(training: boolean): AiLevel[] {
  return training ? [...PLAYER_LEVELS, 'Training'] : [...PLAYER_LEVELS];
}
export const offeredLevels = (): AiLevel[] => levelsFor(trainingOffered());
/** Stable across releases: the previous lobby had no difficulty preference to carry forward. */
export const LOBBY_DIFFICULTY_KEY = 'legatum.lobby.difficulty';

interface DifficultyPreference { level: AiLevel; seats?: Record<string, AiLevel> }
const isLevel = (value: unknown): value is AiLevel => offeredLevels().includes(value as AiLevel);
/** A stored Training (or the older `HardSearch`) reads as Hard unless the developer switch is on. */
const storedLevel = (value: unknown): unknown => value === 'HardSearch' || value === 'Training' ? (trainingOffered() ? 'Training' : 'Hard') : value;

export function readDifficultyPreference(): DifficultyPreference | null {
  try {
    const raw = globalThis.localStorage?.getItem(LOBBY_DIFFICULTY_KEY);
    if (raw) {
      const value = JSON.parse(raw) as DifficultyPreference;
      const level = storedLevel(value?.level);
      if (value && isLevel(level)) return {
        level,
        seats: Object.fromEntries(Object.entries(value.seats ?? {}).map(([seat, l]) => [seat, storedLevel(l)]).filter(([, l]) => isLevel(l))),
      };
    }
  } catch { /* Storage can be unavailable. */ }
  return null;
}

export function rememberDifficulty(lobby: Lobby): boolean {
  const seats: Record<string, AiLevel> = Object.fromEntries(Object.entries(readDifficultyPreference()?.seats ?? {})
    .filter(([key]) => !key.startsWith(`${lobby.mode}:`)));
  lobby.slots.forEach((slot, i) => {
    if (slot.kind === 'ai' && levelOf(slot) !== levelOf(lobby)) seats[`${lobby.mode}:${i}`] = levelOf(slot);
  });
  try {
    if (!globalThis.localStorage) return false;
    globalThis.localStorage.setItem(LOBBY_DIFFICULTY_KEY, JSON.stringify({ level: levelOf(lobby), seats }));
    return true;
  } catch { return false; }
}

export const DIFFICULTY_LABEL: Record<AiLevel, string> = {
  Easy: 'Easy',
  Normal: 'Normal',
  Hard: 'Hard',
  Training: 'Training (slower turns)',
};

/** The closed select and the narrow all-AI control show the short name. */
export const DIFFICULTY_SHORT: Record<AiLevel, string> = { ...DIFFICULTY_LABEL, Training: 'Training' };

export const DIFFICULTY_INFO: Record<AiLevel, string> = {
  Easy: 'Timid and inattentive: smaller armies, attacks only with a big edge, and misses some of its best moves.',
  Normal: 'The standard AI.',
  Hard: 'Keeps a bigger army and plans its wars and scoring. No bonuses: it just decides better.',
  Training: 'A research tool, not a fair opponent. It plays Hard, but it studies each battle using the true positions of nearby units, including ones it cannot see. Turns with battles can take up to 15 s. Every battle is saved.',
};

/**
 * The level a seat or the lobby default is set to. Training only counts on top of Hard, and only
 * with the developer switch: without it a lobby that says Training (a saved or online room) is Hard.
 */
export function levelOf(s: { difficulty: AiDifficulty; search?: boolean }, training = trainingOffered()): AiLevel {
  return training && s.difficulty === 'Hard' && s.search ? 'Training' : s.difficulty;
}

/** The fields a level sets on a seat or the lobby default. */
export function levelFields(level: AiLevel, training = trainingOffered()): { difficulty: AiDifficulty; search: boolean } {
  return level === 'Training' ? { difficulty: 'Hard', search: training } : { difficulty: level, search: false };
}

/** `default` is the empire count a new free-for-all lobby starts with, and what a size change raises a too-small count to. */
export const MAP_SIZES: Record<MapSizeId, { width: number; height: number; min: number; max: number; default: number }> = {
  Small: { width: 40, height: 26, min: 2, max: 4, default: 4 },
  Standard: { width: 48, height: 32, min: 5, max: 8, default: 6 },
  Large: { width: 60, height: 40, min: 8, max: 8, default: 8 },
};

export function mapDimensions(size: MapSize): { width: number; height: number } {
  return typeof size === 'string' ? MAP_SIZES[size] : size;
}

export function mapSizeLabel(size: MapSize): string {
  return typeof size === 'string' ? size : `Custom ${size.width}×${size.height}`;
}

function defaultName(kind: SlotKind, index: number): string {
  if (kind === 'ai') return `AI ${index + 1}`;
  if (kind === 'closed') return '';
  return `Player ${index + 1}`;
}

function isDefaultName(name: string): boolean {
  return /^(AI|Player) \d+$/.test(name) || name === '';
}

const humanKind = (k: SlotKind) => k === 'you' || k === 'human';
const isActive = (s: Slot) => s.kind !== 'closed';

/** Seat kinds offered in a mode (the first seat is always `you`). */
export function slotKinds(mode: LobbyMode): SlotKind[] {
  return mode === 'solo' ? ['ai', 'closed'] : ['human', 'ai', 'closed'];
}

function ai(i: number, team: number, level: AiLevel): Slot {
  const { difficulty, search } = levelFields(level);
  return { kind: 'ai', name: defaultName('ai', i), faction: 'Random', team, difficulty, ...(search ? { search } : {}) };
}

/**
 * Rebuild the seats for a preset, keeping the humans' names and factions. Free-for-all has the
 * map size's default number of empires; Teams (four teams of two) and Co-op (two humans and six
 * AIs) keep all eight seats, since their structure needs them.
 */
export function applyPreset(l: Lobby, preset: PresetId): Lobby {
  if (preset === 'custom') return { ...l, preset };
  // Co-op: two humans on one team would face six uncoordinated AIs, so the AIs start allied in pairs.
  const aiAlliances = preset === 'coop';
  const humans = l.mode === 'solo' ? 1 : 2;
  const keep = l.slots.filter((s) => humanKind(s.kind));
  const slots: Slot[] = [];
  const seats = preset === 'ffa' ? MAP_SIZES[typeof l.mapSize === 'string' ? l.mapSize : 'Standard'].default : MAX_PLAYERS;
  for (let i = 0; i < seats; i++) {
    const team = preset === 'ffa' ? i : preset === 'teams' ? i % 4 : i < Math.max(2, humans) ? 0 : i - Math.max(2, humans) + 1;
    if (i < humans) {
      const prev = keep[i];
      const kind: SlotKind = i === 0 ? 'you' : 'human';
      slots.push({
        kind,
        name: prev?.name ?? defaultName(kind, i),
        faction: prev?.faction ?? (FACTIONS[i % FACTIONS.length] as Faction),
        team,
        difficulty: l.difficulty,
      });
    } else slots.push(ai(i, team, levelOf(l)));
  }
  return { ...l, preset, aiAlliances, slots, mapSize: typeof l.mapSize === 'string' ? recommendedSize(slots.length) : l.mapSize };
}

export function newLobby(mode: LobbyMode, seed: string): Lobby {
  const preference = readDifficultyPreference();
  const base: Lobby = { mode, preset: 'custom', slots: [], mapSize: 'Standard', turnLimit: null, barbarians: true, seed, ...levelFields(preference?.level ?? 'Hard') };
  const lobby = applyPreset(base, mode === 'solo' ? 'ffa' : 'coop');
  const restored = { ...lobby, slots: lobby.slots.map((slot, i) => {
    const override = preference?.seats?.[`${mode}:${i}`];
    return slot.kind === 'ai' && isLevel(override) ? { ...slot, ...levelFields(override) } : slot;
  }) };
  rememberDifficulty(restored);
  return restored;
}

/** Edit one seat. Changing who sits there or their team makes the preset Custom. */
export function setSlot(l: Lobby, i: number, patch: Partial<Slot>): Lobby {
  const slots = l.slots.map((s, j) => {
    if (j !== i) return s;
    const next = { ...s, ...(patch.kind === 'ai' && s.kind !== 'ai' ? levelFields(levelOf(l)) : {}), ...patch };
    if (patch.kind && patch.kind !== s.kind && isDefaultName(s.name)) next.name = defaultName(patch.kind, i);
    return next;
  });
  const structural = patch.kind !== undefined || patch.team !== undefined;
  return { ...l, slots, preset: structural ? 'custom' : l.preset };
}

/** Add an AI seat on its own new team. */
export function addSlot(l: Lobby): Lobby {
  if (l.slots.length >= MAX_PLAYERS) return l;
  const used = new Set(l.slots.map((s) => s.team));
  let team = 0;
  while (used.has(team) && team < MAX_PLAYERS - 1) team++;
  return { ...l, preset: 'custom', slots: [...l.slots, ai(l.slots.length, team, levelOf(l))] };
}

export function removeSlot(l: Lobby, i: number): Lobby {
  if (i === 0 || l.slots.length <= MIN_SEATS) return l;
  // Network reservations belong to stable slots. Never shift another player's slot ID.
  if (l.mode === 'host') return setSlot(l, i, { kind: 'closed' });
  return { ...l, preset: 'custom', slots: l.slots.filter((_, j) => j !== i) };
}

/** Set the default difficulty and apply it to every AI seat. */
export function setDifficulty(l: Lobby, level: AiLevel): Lobby {
  const fields = levelFields(level);
  return { ...l, ...fields, slots: l.slots.map((s) => (s.kind === 'ai' ? { ...s, ...fields } : s)) };
}

export function activeCount(l: Lobby): number {
  return l.slots.filter(isActive).length;
}

function recommendedSize(players: number): MapSizeId {
  return players <= MAP_SIZES.Small.max ? 'Small' : 'Standard';
}

/** An AI seat the player has changed from its defaults: a chosen faction or a name of their own. */
function isCustomised(s: Slot): boolean {
  return s.faction !== 'Random' || !isDefaultName(s.name);
}

/**
 * Choose a map size and snap the number of empires into its recommended range: trim to the
 * default when every AI seat still has its default name and Random faction. Otherwise trim to the
 * maximum, or, below the minimum, raise to the size's default count (a count already in range is left alone). Trimming takes default AI seats from the bottom, never a
 * human seat, a customised AI (named or with a chosen faction), or a seat whose removal would
 * leave fewer than two teams; if only those are left the count stays and `notes` warns.
 * Raising appends default AI seats (reopening a closed seat when all eight are used). In a
 * network room trimmed seats are closed rather than removed, so no seat's id shifts.
 * Afterwards the player may still go outside the range with the × and add-seat controls.
 */
export function setMapSize(l: Lobby, size: MapSizeId): Lobby {
  let next: Lobby = { ...l, mapSize: size };
  const { min, max, default: target } = MAP_SIZES[size];
  const aiSeats = l.slots.filter((s) => s.kind === 'ai');
  const untouched = aiSeats.length > 0 && aiSeats.every((s) => !isCustomised(s));
  const ceiling = untouched ? target : max;
  while (activeCount(next) > ceiling) {
    let cut = -1;
    for (let i = next.slots.length - 1; i > 0; i--) {
      const s = next.slots[i];
      if (s.kind !== 'ai' || isCustomised(s)) continue;
      const without = next.mode === 'host' ? setSlot(next, i, { kind: 'closed' }) : { ...next, slots: next.slots.filter((_, j) => j !== i) };
      if (teams(without).length < 2) continue;
      cut = i;
      break;
    }
    if (cut < 0) break;
    next = removeSeatForSize(next, cut);
  }
  if (activeCount(next) >= (untouched ? target : min)) return next;
  while (activeCount(next) < target) {
    if (next.slots.length < MAX_PLAYERS) next = addSlot(next);
    else {
      const closed = next.slots.findIndex((s) => s.kind === 'closed');
      if (closed < 0) break;
      next = setSlot(next, closed, { kind: 'ai', ...levelFields(levelOf(next)), name: defaultName('ai', closed), faction: 'Random' });
    }
  }
  return next;
}

function removeSeatForSize(l: Lobby, i: number): Lobby {
  if (l.mode === 'host') return { ...setSlot(l, i, { kind: 'closed' }), preset: 'custom' };
  return { ...l, preset: 'custom', slots: l.slots.filter((_, j) => j !== i) };
}

function sizeFits(size: MapSizeId, players: number): boolean {
  const s = MAP_SIZES[size];
  return players >= s.min && players <= s.max;
}

/**
 * Slot indices in player-id order: the humans first (the network server expects human seats
 * first), then the AIs, each group in seat order. Closed seats are left out.
 */
function playerOrder(l: Lobby): number[] {
  const idx = l.slots.map((_, i) => i).filter((i) => isActive(l.slots[i]));
  return [...idx.filter((i) => humanKind(l.slots[i].kind)), ...idx.filter((i) => !humanKind(l.slots[i].kind))];
}

const seedKnown = (seed: string) => { try { BigInt(seed); return true; } catch { return false; } };

/**
 * The faction whose colour each seat will wear in game: its own, or a free one when two seats share
 * a faction. Null for closed seats, and for Random seats when the seed is hidden (a guest's view of
 * a network room), since their faction is not known yet.
 */
export function slotColorFactions(l: Lobby): (Faction | null)[] {
  const resolved = seedKnown(l.seed) ? resolveFactions(l) : l.slots.map((s) => (s.faction === 'Random' ? undefined : s.faction));
  const order = playerOrder(l).filter((slot) => resolved[slot] !== undefined);
  const colored = colorFactions(order.map((slot) => resolved[slot]!));
  const out: (Faction | null)[] = l.slots.map(() => null);
  order.forEach((slot, i) => (out[slot] = colored[i]));
  return out;
}

function resolveFactions(l: Lobby): Faction[] {
  const active = new Set(playerOrder(l));
  const resolved: (Faction | undefined)[] = l.slots.map((s, i) => !active.has(i) || s.faction === 'Random' ? undefined : s.faction);
  const used = new Set(resolved.filter((f): f is Faction => f !== undefined));
  const pool = FACTIONS.filter((f) => !used.has(f));
  for (const slot of playerOrder(l)) {
    if (resolved[slot] !== undefined) continue;
    const counts = FACTIONS.map((f) => resolved.filter((x) => x === f).length);
    const least = Math.min(...counts);
    const available = pool.length ? pool : FACTIONS.filter((_, i) => counts[i] === least);
    const f = available[factionChoiceIndex(l.seed, slot, available.length)];
    resolved[slot] = f;
    if (pool.length) pool.splice(pool.indexOf(f), 1);
  }
  return resolved as Faction[];
}

export interface TeamSummary {
  /** The team id in the game (compacted, in player order). */
  team: number;
  /** The lobby's team number (0-based). */
  lobbyTeam: number;
  slots: number[];
  target: number;
}

/** Teams in play with their members and VP target (the core's scoring::team_target). */
export function teams(l: Lobby): TeamSummary[] {
  const out: TeamSummary[] = [];
  for (const slot of playerOrder(l)) {
    const lobbyTeam = l.slots[slot].team;
    let t = out.find((x) => x.lobbyTeam === lobbyTeam);
    if (!t) {
      t = { team: out.length, lobbyTeam, slots: [], target: 0 };
      out.push(t);
    }
    t.slots.push(slot);
  }
  for (const t of out) t.target = teamTarget(t.slots.length);
  return out;
}

/** Teams made only of AI seats: the ones `aiAlliances` puts into pairs. */
export function aiOnlyTeams(l: Lobby): number {
  return teams(l).filter((t) => t.slots.every((i) => l.slots[i].kind === 'ai')).length;
}

/** True when the setup turns AI alliances on: the same condition `buildSetup` uses. */
export function aiAlliancesOn(l: Lobby): boolean {
  return !!l.aiAlliances && aiOnlyTeams(l) >= 2;
}

/** Problems that block starting the game, locally or on the online host. */
export function problems(l: Lobby): string[] {
  const p: string[] = [];
  const active = l.slots.filter(isActive);
  if (!validTurnLimit(l.turnLimit)) p.push('Turn limit must be a whole number between 1 and 4,294,967,294, or Off.');
  if (active.length < MIN_SEATS) p.push('A game needs at least two empires.');
  if (teams(l).length < 2) p.push('Everyone is on one team: put at least one empire on another team.');
  if (active.some((s) => humanKind(s.kind) && !s.name.trim())) p.push('Every human seat needs a name.');
  if (!/^\d{1,20}$/.test(l.seed) || BigInt(l.seed) >= 2n ** 64n) p.push('The map seed must be a whole number.');
  if (l.mode === 'hotseat' && active.filter((s) => humanKind(s.kind)).length < 2) p.push('Hot seat needs at least two human seats.');
  return p;
}

/** Advice that doesn't block starting. */
export function notes(l: Lobby): string[] {
  if (typeof l.mapSize !== 'string') return [];
  const n = activeCount(l);
  if (sizeFits(l.mapSize, n)) return [];
  const s = MAP_SIZES[l.mapSize];
  const rec = s.min === s.max ? `${s.min}` : `${s.min}-${s.max}`;
  return [`${l.mapSize} maps suit ${rec} empires; you have ${n}. ${recommendedSize(n)} is recommended.`];
}

export function buildSetup(l: Lobby): GameSetup {
  const ts = teams(l);
  const factions = resolveFactions(l);
  const teamOf = (slot: number) => ts.find((t) => t.slots.includes(slot))!.team;
  const players: PlayerSetup[] = playerOrder(l).map((slot) => {
    const s = l.slots[slot];
    const p: PlayerSetup = {
      name: s.name.trim() || defaultName(s.kind, slot),
      faction: factions[slot],
      team: teamOf(slot),
      is_human: humanKind(s.kind),
    };
    if (s.kind === 'ai' && s.difficulty !== 'Normal') p.difficulty = s.difficulty;
    if (s.kind === 'ai' && levelOf(s) === 'Training') p.search = true;
    return p;
  });
  const size = mapDimensions(l.mapSize);
  return { seed: l.seed, width: size.width, height: size.height, players, turn_limit: l.turnLimit, barbarians: l.barbarians !== false, ...(aiAlliancesOn(l) ? { ai_alliances: true } : {}) };
}
