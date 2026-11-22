// Byzantium's Heir on the map and in text: his position is public every turn (`UnitView.heir_of`, `.owner`, `.pos` are always in
// the view, fog and unexplored tiles included), so the map marks him wherever he stands and says who holds him.
import { isBarbarian } from './barbarians';
import { factionName } from './data';
import type { PlayerView, UnitView } from './types';

export const isHeir = (u: Pick<UnitView, 'heir_of'>) => u.heir_of !== undefined && u.heir_of !== null;

const teamOf = (view: PlayerView, id: number) => view.players.find((p) => p.id === id)?.team;
const nameOf = (view: PlayerView, id: number) => {
  const f = view.players.find((p) => p.id === id)?.faction;
  return f ? factionName(f) : 'another empire';
};

/** The short word under his token: "Your Heir", "Heir to the Throne", "You hold him", "Held by Sparta", "Held by barbarians". */
export function heirTag(view: PlayerView, u: UnitView): string | null {
  if (!isHeir(u)) return null;
  const holder = u.owner, goal = u.heir_of!;
  const mine = (id: number) => teamOf(view, id) === view.team;
  if (isBarbarian(holder)) return 'Held by barbarians';
  if (holder === goal) return mine(goal) ? 'Your Heir' : 'Heir to the Throne';
  return mine(holder) ? 'You hold him' : `Held by ${nameOf(view, holder)}`;
}

/** The tile-card's status line; the card head already names the Heir. `vp` is the goal's points. */
export function heirHoverLine(view: PlayerView, u: UnitView, vp: number): string | null {
  if (!isHeir(u)) return null;
  const holder = u.owner, goal = u.heir_of!;
  const mine = (id: number) => teamOf(view, id) === view.team;
  if (isBarbarian(holder)) return 'Held by barbarians: nobody scores until he is freed';
  if (mine(holder)) return holder === goal ? `Hold him to score ${vp} VP, and keep him safe` : `You hold him: you score ${vp} VP; a rival that captures him takes them`;
  return `${holder === goal ? '' : `Held by ${nameOf(view, holder)}: `}Be at War, then capture him to score ${vp} VP`;
}
