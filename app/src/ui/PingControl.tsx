import { CrestMedallion } from '../brand/ornaments';
import { useGame } from '../store';
import { playerColor } from '../theme';
import { pingsFor } from '../teamPing';
import './pingControl.css';

/**
 * The team Ping control, at the foot of the right sidebar: "Ping", then click a tile to mark it for your teammates (P
 * also starts it, Esc cancels). A teammate's live ping gets a button that names them and takes you to it, from anywhere
 * on the map. A solo game has no teammate, so it shows nothing.
 */
export function PingControl() {
  const available = useGame((s) => s.canPing());
  const mode = useGame((s) => s.pingMode);
  const view = useGame((s) => s.views?.[s.seat] ?? null);
  const pings = useGame((s) => s.pings);
  if (!available || !view) return null;
  const theirs = pingsFor(view, pings).filter((p) => p.from !== view.me);
  const set = useGame.getState().setPingMode;
  return (
    <div className="ping-row" data-testid="ping-control" data-incoming={theirs.length ? 'true' : undefined}>
      <button type="button" className={`btn ping-btn${mode ? ' on' : ''}`} aria-pressed={mode} data-testid="ping-button"
        title="Mark a tile for your teammate: click Ping, then the map. Shortcut P; Esc cancels."
        onClick={() => set(!mode)}>
        {mode ? 'Cancel ping' : 'Ping'} <kbd className="kbd">{mode ? 'Esc' : 'P'}</kbd>
      </button>
      {mode && <span className="ping-hint" data-testid="ping-hint" role="status">Click a tile to ping it</span>}
      {theirs.map((p) => {
        const from = view.players.find((x) => x.id === p.from);
        const name = from?.name ?? 'Your teammate';
        return (
          <button key={p.from} type="button" className="btn ping-go" data-testid="ping-go"
            title={`${name} pinged a tile. Go there.`}
            onClick={() => useGame.getState().jumpTo(p.hex)}>
            {from && <CrestMedallion faction={from.faction} color={playerColor(from.id)} size={18} />}
            <span>{name} pinged · Go to ping</span>
          </button>
        );
      })}
    </div>
  );
}
