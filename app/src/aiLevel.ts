import type { PlayerPublic } from './types';

/** The level recorded in the game setup, including saves made before the view exposed it. */
export function aiLevel(player: Pick<PlayerPublic, 'is_human' | 'difficulty' | 'search'>): string | null {
  if (player.is_human) return null;
  return player.difficulty === 'Hard' && player.search ? 'Training' : player.difficulty ?? 'Normal';
}
