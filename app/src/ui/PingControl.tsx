import { useGame } from '../store';
import { pingsFor } from '../teamPing';
import './pingControl.css';

/**
 * The team Ping control, under the minimap: "Ping", then click a tile to mark it for your teammates (P also starts it,
 * Esc cancels). A teammate's live ping gets a button that takes you to it. A solo game has no teammate, so it shows nothing.
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
    <div className="ping-row" data-testid="ping-control">
      <button type="button" className={`btn ping-btn${mode ? ' on' : ''}`} aria-pressed={mode} data-testid="ping-button"
        title="Mark a tile for your teammate: click Ping, then the map. Shortcut P; Esc cancels."
        onClick={() => set(!mode)}>
        {mode ? 'Click a tile' : 'Ping'}
      </button>
      {theirs.map((p) => (
        <button key={p.from} type="button" className="btn quiet ping-go" data-testid="ping-go"
          title={`${view.players.find((x) => x.id === p.from)?.name ?? 'Your teammate'} pinged a tile. Go there.`}
          onClick={() => useGame.getState().jumpTo(p.hex)}>
          Go to ping
        </button>
      ))}
    </div>
  );
}
