// Build GameSetup JSON for the engine in TypeScript (the engine accepts any setup: 2-8 players,
// any mix of humans and AIs, any teams, any map size).
import { FACTIONS } from './data';
import type { Faction, GameSetup, PlayerSetup } from './types';

const STANDARD_WIDTH = 48;
const STANDARD_HEIGHT = 32;
export const MAX_PLAYERS = 8;

/** Seeded choice from an available faction pool, stable across runs and platforms. */
export function factionChoiceIndex(seed: string, i: number, size: number): number {
  const range = 1n << 64n;
  const limit = range - range % BigInt(size);
  for (let attempt = 0n; ; attempt++) {
    let x = BigInt.asUintN(64, BigInt(seed) ^ (BigInt(i + 1) * 0x9e3779b97f4a7c15n) ^ (attempt * 0xd1b54a32d192ed03n));
    x ^= x >> 30n; x = BigInt.asUintN(64, x * 0xbf58476d1ce4e5b9n);
    x ^= x >> 27n; x = BigInt.asUintN(64, x * 0x94d049bb133111ebn);
    x ^= x >> 31n;
    if (x < limit) return Number(x % BigInt(size));
  }
}

function aiFaction(seed: string, i: number, pool: Faction[]): Faction {
  return pool[(Number(BigInt(seed) % BigInt(pool.length)) + i) % pool.length];
}

export interface HumanSeat {
  name: string;
  faction: Faction;
}

/**
 * Humans first (one shared team, ids 0..h-1), then AIs up to `total` players, each AI on its own
 * team. One human plays solo; several humans join through the online lobby.
 */
export function coopVsAiSetup(seed: string, humans: HumanSeat[], total = MAX_PLAYERS): GameSetup {
  if (humans.length < 1) throw new Error('at least one human seat');
  if (total < 2 || total > MAX_PLAYERS || total < humans.length) throw new Error(`bad player count ${total}`);
  const players: PlayerSetup[] = humans.map((h) => ({ name: h.name, faction: h.faction, team: 0, is_human: true }));
  const pool = FACTIONS.filter((f) => !humans.some((h) => h.faction === f));
  for (let i = 0; players.length < total; i++) {
    players.push({ name: `AI ${i + 1}`, faction: aiFaction(seed, i, pool), team: i + 1, is_human: false });
  }
  return { seed, width: STANDARD_WIDTH, height: STANDARD_HEIGHT, players, turn_limit: null };
}

/**
 * The solo VP target the core uses (RuleExperiments::solo_vp_target under VP v2): the permanent
 * ceiling plus one, capped at 10. The seven-era tree gives 10. A started
 * game's standings carry the real number (store.test.ts checks this against the engine).
 */
const SOLO_VP_TARGET = 10;

/** VP target per team (scoring::team_target under VP v2): the solo target per member. */
export function teamTarget(members: number): number {
  return SOLO_VP_TARGET * Math.max(1, members);
}
