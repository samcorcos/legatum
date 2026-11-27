import { useEffect } from 'react';
import { useGame } from '../store';
import { HexBackdrop } from './HexBackdrop';
import { OpenGamesList } from '../ui/OpenGamesList';

/** Online play: the games other players are hosting. The list refreshes by itself while this is open. */
export function OnlineLobbyScreen() {
  const online = useGame((s) => s.online);
  const goto = useGame((s) => s.goto);

  // Esc goes back, like the other screens that have a Back button.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !e.defaultPrevented) goto('title');
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [goto]);

  if (!online?.relay) return null;
  return (
    <div className="setup-screen">
      <HexBackdrop />
      <div className="setup-inner join">
        <div className="setup-top">
          <button className="btn quiet" onClick={() => goto('title')}>
            ← Back
          </button>
          <h2>Hosted games</h2>
          <div />
        </div>
        <p className="setup-lede">Pick a game to join. The list updates by itself.</p>
        <OpenGamesList />
      </div>
    </div>
  );
}
