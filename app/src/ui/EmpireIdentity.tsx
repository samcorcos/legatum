import { CrestMedallion } from '../brand/ornaments';
import { describeLocation, empireLocation, empireMembers } from '../empires';
import { useGame } from '../store';
import { playerColor, playerInk } from '../theme';
import type { PlayerView, TeamId } from '../types';
import { PlayerBadge } from './PlayerBadge';

/** The empire's crest in its map color with its seat number: the same mark as its victory-race row. */
export function EmpireMark({ view, team, size = 26 }: { view: PlayerView; team: TeamId; size?: number }) {
  return <span className="empire-mark" aria-hidden="true">
    {empireMembers(view, team).map((m) => <span className="empire-mark-member" key={m.id} style={{ ['--pc' as string]: playerInk(m.id) }}>
      <CrestMedallion faction={m.faction} color={playerColor(m.id)} size={size} />
      <PlayerBadge id={m.id} />
    </span>)}
  </span>;
}

/** Where the viewer knows this empire to be, and a button that pans the map there. */
export function EmpireWhere({ view, team }: { view: PlayerView; team: TeamId }) {
  const loc = empireLocation(view, team);
  return <span className="empire-where">
    <span className="empire-where-text">{describeLocation(loc)}</span>
    {loc && <button className="link empire-show" onClick={() => {
      const game = useGame.getState();
      game.jumpTo(loc.at);
      // The inbox covers the map's centre (the whole map on a phone): close it so the empire is in
      // view. The pending offer keeps its "Respond to offer" call to action for the way back.
      game.set({ diplomacyOpen: false });
    }}>Show on map</button>}
  </span>;
}
