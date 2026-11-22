import { useCallback, useEffect, useState } from 'react';
import { useGame } from '../store';
import { HexBackdrop } from './HexBackdrop';
import {
  LOBBY_POLL_MS,
  fetchLobby,
  phaseText,
  seatsText,
  versionProblem,
  type RoomInfo,
} from '../platform/relayLobby';

type Load = { kind: 'loading' } | { kind: 'ready'; rooms: RoomInfo[] } | { kind: 'failed' };

/** Online play: the games other players are hosting. The list refreshes by itself while this is open. */
export function OnlineLobbyScreen() {
  const online = useGame((s) => s.online);
  const goto = useGame((s) => s.goto);
  const joinOnline = useGame((s) => s.joinOnline);
  const [load, setLoad] = useState<Load>({ kind: 'loading' });
  const [joining, setJoining] = useState<string | null>(null);
  const [joinError, setJoinError] = useState<{ room: string; text: string } | null>(null);
  const relay = online?.relay ?? null;

  useEffect(() => {
    if (!relay) return;
    let live = true;
    const refresh = () =>
      fetchLobby(relay).then(
        (rooms) => live && setLoad({ kind: 'ready', rooms }),
        () => live && setLoad({ kind: 'failed' }),
      );
    void refresh();
    const timer = setInterval(refresh, LOBBY_POLL_MS);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [relay]);

  const join = useCallback(
    async (room: RoomInfo) => {
      if (!relay || joining) return;
      setJoining(room.room);
      setJoinError(null);
      const problem = await joinOnline(relay, room.room);
      setJoining(null);
      if (problem) setJoinError({ room: room.room, text: problem });
    },
    [relay, joining, joinOnline],
  );

  // Esc goes back, like the other screens that have a Back button.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !e.defaultPrevented) goto('title');
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [goto]);

  if (!online || !relay) return null;
  return (
    <div className="setup-screen">
      <HexBackdrop />
      <div className="setup-inner join">
        <div className="setup-top">
          <button className="btn quiet" onClick={() => goto('title')}>
            ← Back
          </button>
          <h2>Open games</h2>
          <div />
        </div>
        <p className="setup-lede">Pick a game to join. The list updates by itself.</p>
        {load.kind === 'loading' && (
          <div className="waiting-host" role="status">
            <div className="spinner" />
            Looking for games…
          </div>
        )}
        {load.kind === 'failed' && (
          <div className="online-empty" role="alert" data-state="error">
            <p>The list of games could not be loaded.</p>
            <p className="online-empty-sub">
              <span className="spinner" /> Trying again…
            </p>
          </div>
        )}
        {load.kind === 'ready' && load.rooms.length === 0 && (
          <div className="online-empty" data-state="empty">
            <p>No games are being hosted right now.</p>
            <p className="online-empty-sub">Go back and choose Host Game to start one.</p>
          </div>
        )}
        {load.kind === 'ready' && load.rooms.length > 0 && (
          <ul className="online-rooms">
            {load.rooms.map((room) => {
              const mismatch = versionProblem(room, online);
              const full = room.open_seats <= 0;
              const note = mismatch ?? (joinError?.room === room.room ? joinError.text : null);
              return (
                <li key={room.room} className={`online-room ${mismatch ? 'blocked' : ''}`}>
                  <div className="online-room-main">
                    {note && (
                      <div className="online-room-note" role="alert">
                        <span className="online-room-mark" aria-hidden>!</span>
                        {note}
                      </div>
                    )}
                    <div className="online-room-name">{room.name || 'Unnamed game'}</div>
                    <div className="online-room-meta">
                      <span>{seatsText(room)}</span>
                      <span className="online-room-phase">{phaseText(room)}</span>
                    </div>
                  </div>
                  <button
                    className="btn primary online-join"
                    disabled={!!mismatch || full || !!joining}
                    onClick={() => join(room)}
                    aria-label={joining === room.room ? undefined : `Join ${room.name || 'this game'}`}
                  >
                    {joining === room.room ? 'Joining…' : 'Join'}
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
