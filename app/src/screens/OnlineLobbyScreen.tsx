import { useCallback, useEffect, useState } from 'react';
import { useGame } from '../store';
import { HexBackdrop } from './HexBackdrop';
import {
  LOBBY_POLL_MS,
  fetchLobby,
  phaseText,
  roomBase,
  seatsText,
  versionProblem,
  type RoomInfo,
} from '../platform/relayLobby';
import { buildForRoom, buildUrl, navigate } from '../platform/versions';
import { useSiteBuilds } from '../ui/useSiteBuilds';
import { keptSeat, rememberSeat } from '../remote';

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
  const builds = useSiteBuilds();

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
      const problem = await joinOnline(relay, room.room, room.locked);
      setJoining(null);
      if (problem) {
        setJoinError({ room: room.room, text: problem });
        return;
      }
      // This device's own seat in a game under way: take it straight back, as a reload does.
      const kept = room.phase === 'playing' ? keptSeat(roomBase(relay, room.room)) : null;
      const g = useGame.getState();
      if (kept && g.screen === 'join') {
        rememberSeat({ seat: kept.seat, name: kept.name, gameId: null, token: kept.token });
        g.joinSeat(kept.seat, kept.name);
      }
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
              // A game on another version the site still serves is joined there: Join opens that version.
              const other = versionProblem(room, online) ? buildForRoom(builds, room) : null;
              const mismatch = other ? null : versionProblem(room, online);
              // A seat this device held (before a restart) can be taken back even when the game shows as full.
              const yours = !!keptSeat(roomBase(relay, room.room));
              const full = room.open_seats <= 0 && !yours;
              const note = mismatch ?? (joinError?.room === room.room ? joinError.text : other ? 'This game is on another version. Joining opens that version.' : null);
              return (
                <li key={room.room} className={`online-room ${mismatch ? 'blocked' : ''}`}>
                  <div className="online-room-main">
                    {note && (
                      <div className="online-room-note" role={other && !mismatch && joinError?.room !== room.room ? 'status' : 'alert'}>
                        {other && !mismatch && joinError?.room !== room.room
                          ? <span className="online-room-mark info" aria-hidden>→</span>
                          : <span className="online-room-mark" aria-hidden>!</span>}
                        {note}
                      </div>
                    )}
                    <div className="online-room-name">{room.name || 'Unnamed game'}</div>
                    <div className="online-room-meta">
                      <span>{seatsText(room)}</span>
                      {yours && <span className="online-room-yours">Your seat is here</span>}
                      <span className="online-room-phase">{phaseText(room)}</span>
                      {room.locked && (
                        <span className="online-room-lock">
                          <svg viewBox="0 0 12 14" width="11" height="13" aria-hidden focusable="false">
                            <rect x="1.5" y="6" width="9" height="7" rx="1" fill="currentColor" />
                            <path d="M3.5 6V4a2.5 2.5 0 0 1 5 0v2" fill="none" stroke="currentColor" strokeWidth="1.6" />
                          </svg>
                          Locked
                        </span>
                      )}
                    </div>
                  </div>
                  <button
                    className="btn primary online-join"
                    disabled={!!mismatch || full || !!joining}
                    onClick={() => (other ? navigate(buildUrl(other.id, { join: room.room })) : join(room))}
                    aria-label={joining === room.room ? undefined : `${yours && room.phase === 'playing' ? 'Rejoin' : 'Join'} ${room.name || 'this game'}`}
                  >
                    {joining === room.room ? 'Joining…' : yours && room.phase === 'playing' ? 'Rejoin' : 'Join'}
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
