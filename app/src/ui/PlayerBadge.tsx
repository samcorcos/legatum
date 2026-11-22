import { playerColor } from '../theme';
import { Icon } from './bits';

/** Player IDs, unlike faction crests and colors, remain unambiguous in every look. */
export function PlayerBadge({ id, name }: { id: number; name?: string }) {
  if (id === 255) return <span className="player-badge barbarian" role="img" style={{ background: playerColor(id) }} aria-label="Barbarians"><Icon name="ui-cross" size={10} color="currentColor" /></span>;
  return <span className="player-badge" role="img" style={{ background: playerColor(id) }} aria-label={`Player ${id + 1}${name ? `: ${name}` : ''}`}>{id + 1}</span>;
}
