// A network player's view of the host's staging room: take an open seat, pick your name and
// faction in that seat's own row, Ready up from the dock, and wait for the host to start.
import { useEffect, useState } from 'react';
import { useGame } from '../store';
import { Icon } from '../ui/bits';
import { JoinCodeField } from '../ui/JoinCodeField';
import { connectionFirst } from '../platform/relayLobby';
import { rememberPlayerName, savedPlayerName } from '../platform/online';
import { canTakeSeat, everyoneReady, lobbyFromStaging } from '../staging';
import { LobbyFrame, Victory } from './LobbyScreen';
import { SeatsTable } from './lobby/SeatsTable';
import { Settings } from './lobby/Settings';

export function StagingScreen() {
  const staging = useGame((s) => s.staging);
  const mySeat = useGame((s) => s.stagingSeat);
  const connected = useGame((s) => s.stagingConnected);
  const leaving = useGame((s) => s.stagingLeaving);
  const error = useGame((s) => s.error);
  const locked = useGame((s) => !!s.net?.locked);
  const codeText = useGame((s) => s.joinCodeText);
  const g = useGame.getState;
  // The name typed on the Multiplayer menu is the name this player sits down with.
  const [name, setName] = useState(() => savedPlayerName()?.slice(0, 24) || 'Player');
  useEffect(() => {
    if (mySeat !== null && staging?.occupants[mySeat]) setName(staging.occupants[mySeat]!.name);
  }, [mySeat, staging?.room_id]);

  useEffect(() => {
    useGame.getState().enterStaging();
  }, []);

  const back = () => {
    g().exitStaging();
    g().goto('title');
  };

  if (!staging) {
    return (
      <LobbyFrame title="Game room" onBack={back} error={error}>
        <div className="waiting-host staging-wait">
          <div className="spinner" />
          Waiting for the host to open a game room…
        </div>
      </LobbyFrame>
    );
  }

  const lobby = lobbyFromStaging(staging);
  const me = mySeat !== null ? staging.occupants[mySeat] : null;
  const allReady = everyoneReady(staging);
  const host = staging.occupants[0]?.name;
  const leaveReason = !connected ? connectionFirst('leave')
    : leaving ? 'Leaving your seat…' : null;

  const renderOpen = (seat: number) => {
    const o = staging.occupants[seat];
    if (seat === mySeat) {
      return (
        <span className="my-seat">
          <input
            className="lobby-input"
            value={name}
            maxLength={24}
            aria-label="Your name"
            onChange={(e) => {
              setName(e.target.value);
              rememberPlayerName(e.target.value);
              g().updateStagingSeat({ name: e.target.value });
            }}
          />
        </span>
      );
    }
    if (o && !canTakeSeat(staging, seat)) {
      return (
        <span className={`occupant ${o.ready ? 'ready' : ''}`}>
          <span className="occupant-name">{o.name}</span>
          <span className="occupant-status">{!o.connected ? 'Reserved — ask the host to open this seat' : o.ready ? <><Icon name="ui-check" size={12} className="ui-mark" /> Ready</> : 'not ready'}</span>
        </span>
      );
    }
    return (
      <button className="btn primary take-seat" disabled={mySeat !== null || (locked && !codeText)} onClick={() => g().takeStagingSeat(seat, name.trim() || 'Player')}>
        Take seat
      </button>
    );
  };

  // The dock holds the one action this player owes: take a seat, then Ready.
  const dock = (
    <>
      <div className="lobby-dock-status" role="status">
        <div className="lobby-dock-note">
          {mySeat === null
            ? 'Take an open seat to play.'
            : allReady
              ? 'Everyone is Ready. Waiting for the host to start…'
              : me?.ready
                ? 'Waiting for the other players to be Ready.'
                : 'Choose your faction in your row, then press Ready.'}
        </div>
        {leaveReason && <div id="leave-seat-reason">{leaveReason}</div>}
      </div>
      {mySeat !== null && me && (
        <>
          <button className="btn quiet" disabled={!!leaveReason} aria-describedby={leaveReason ? 'leave-seat-reason' : undefined} onClick={() => g().leaveStagingSeat()}>
            Leave seat
          </button>
          <button className={`btn big lobby-go ${me.ready ? '' : 'primary'}`} onClick={() => g().setStagingReady(!me.ready)}>
            {me.ready ? <><Icon name="ui-check" size={14} className="ui-mark" /> Ready</> : 'Ready'}
            <span className="btn-sub">{me.ready ? 'Click to change your mind' : 'Lock in your seat and faction'}</span>
          </button>
        </>
      )}
    </>
  );

  return (
    <LobbyFrame title="Game room" subtitle={`${host ? `Hosted by ${host}` : 'You joined this game'}. The host sets the map and starts the game.`} onBack={back} error={error} dock={dock}>
      {locked && mySeat === null && <JoinCodeField />}
      <div className="lobby-body">
        <SeatsTable
          lobby={lobby}
          room={{ state: staging, guest: true, mySeat, renderOpen, onMyFaction: (faction) => g().updateStagingSeat({ faction }) }}
        />
        <aside className="lobby-side">
          <Settings lobby={lobby} />
          <Victory lobby={lobby} />
        </aside>
      </div>
    </LobbyFrame>
  );
}
