import { useEffect, useRef } from 'react';
import { useGame } from '../store';
import { playerColor } from '../theme';
import { CrestMedallion } from '../brand/ornaments';

/**
 * Hot seat privacy: between two local players' planning, cover the map until the next player
 * takes the device, so nobody sees the other's empire, orders or fog.
 */
export function HandoffScreen() {
  const handoff = useGame((s) => s.handoff);
  const view = useGame((s) => (s.handoff !== null ? s.views?.[s.handoff] : null));
  const button = useRef<HTMLButtonElement>(null);
  useEffect(() => { if (handoff !== null) button.current?.focus({ preventScroll: true }); }, [handoff]);
  if (handoff === null || !view) return null;
  const me = view.players.find((p) => p.id === view.me)!;
  const start = () => useGame.getState().set({ handoff: null });
  return <div className="handoff-screen" role="dialog" aria-modal="true" aria-labelledby="handoff-title" data-game-overlay
    onKeyDown={(e) => { e.stopPropagation(); if (e.key === 'Escape') { e.preventDefault(); start(); } }}>
    <div className="handoff-card">
      <CrestMedallion faction={me.faction} color={playerColor(me.id)} size={72} />
      <h2 id="handoff-title">Pass to {me.name}</h2>
      <p>Turn {view.turn}. The other players have planned; their orders stay hidden.</p>
      <button ref={button} className="btn primary" onClick={start}>I'm {me.name}: start planning</button>
    </div>
  </div>;
}
