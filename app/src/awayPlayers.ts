// Online: the other human players whose computers are not connected right now. Their seats do not hold
// up the turn (the host ends it without them, and their empires get no orders), so the game says who is
// away and asks before a Ready that would end the turn without them.
import type { State } from './store';
import { empireOfPlayer } from './empires';
import { factionName } from './data';

export interface AwayPlayer {
  /** The seat name, e.g. "Alice". */
  name: string;
  /** The empire, named by its faction, e.g. "India". */
  empire: string;
}

export function awayPlayers(s: Pick<State, 'client' | 'net' | 'views' | 'seat'>): AwayPlayer[] {
  if (s.client?.kind !== 'remote' || !s.net || s.net.conn !== 'open') return [];
  const view = s.views?.[s.seat];
  return s.net.seats
    .filter((st) => st.seat !== s.net!.mySeat && !st.connected)
    .filter((st) => view?.players.find((p) => p.id === st.player)?.alive !== false)
    .map((st) => ({
      name: st.name || 'A player',
      empire: view?.players.some((p) => p.id === st.player) ? empireOfPlayer(view, st.player) : factionName(st.faction),
    }));
}

/** "Alice", "Alice and Bo", "Alice, Bo and Cy". */
export function namesList(names: string[]): string {
  return names.length <= 1 ? (names[0] ?? '') : `${names.slice(0, -1).join(', ')} and ${names[names.length - 1]}`;
}

/** One line on who is away and what that means for their empires. */
export function awayLine(away: AwayPlayer[], when: 'until' | 'turn'): string {
  if (!away.length) return '';
  const who = namesList(away.map((a) => a.name));
  const empires = namesList(away.map((a) => a.empire));
  const verb = away.length === 1 ? 'is' : 'are';
  return when === 'until'
    ? `${who} ${verb} not connected. ${empires} get${away.length === 1 ? 's' : ''} no orders until ${away.length === 1 ? who : 'they'} rejoin${away.length === 1 ? 's' : ''}.`
    : `${who} ${verb} not connected, so ${empires} get${away.length === 1 ? 's' : ''} no orders this turn.`;
}
