import { useEffect } from 'react';
import { useGame } from '../store';
import { keptSeat, rememberSeat, rememberedSeat, serverBase } from '../remote';
import { FACTION_INFO } from '../data';
import { FACTION_SHEETS } from '../factionSheets';
import { colorFactions, factionColor } from '../theme';
import { FactionMark } from '../ui/bits';
import { HexBackdrop } from './HexBackdrop';
import { ReclaimSeatButton } from '../ui/ReclaimSeatButton';
import { JoinCodeField } from '../ui/JoinCodeField';

/** An online game: pick which human seat this browser plays. */
export function JoinScreen() {
  const net = useGame((s) => s.net);
  const exitToTitle = useGame((s) => s.exitToTitle);
  const joinSeat = useGame((s) => s.joinSeat);
  const refresh = useGame((s) => s.refreshStatus);
  const busy = useGame((s) => s.busy);
  const error = useGame((s) => s.error);
  const codeText = useGame((s) => s.joinCodeText);
  useEffect(() => {
    refresh();
    const t = setInterval(() => refresh(), 2500);
    return () => clearInterval(t);
  }, [refresh]);
  if (!net) return null;
  // This tab's seat, or else the seat this device held before a restart or a closed tab.
  const kept = rememberedSeat() ? null : keptSeat(serverBase());
  const mine = rememberedSeat() ?? (kept && { seat: kept.seat, name: kept.name, gameId: null, token: kept.token });
  const join = (seat: number, name: string) => {
    // Taking back a kept seat proves it with the kept credential, as a reload does.
    if (kept && kept.seat === seat) rememberSeat({ seat, name: kept.name, gameId: null, token: kept.token });
    joinSeat(seat, name);
  };
  const seats = net.status.seats;
  const colored = colorFactions(seats.map((s) => s.faction));
  return (
    <div className="setup-screen">
      <HexBackdrop />
      <div className="setup-inner join">
        <div className="setup-top">
          <button className="btn quiet" onClick={exitToTitle}>
            ← Back
          </button>
          <h2>Choose your seat</h2>
          <div />
        </div>
        <p className="setup-lede">
          Every player plays on their own computer. Pick an open seat to command that empire. Turn{' '}
          {net.status.turn} is being planned.
        </p>
        {net.locked && <JoinCodeField />}
        {error && <div className="toast bad inline" role="alert">{error}</div>}
        <div className="join-seats">
          {seats.map((s, i) => {
            const info = FACTION_INFO[s.faction];
            const wasMine = mine?.seat === s.seat && !!mine.token && !mine.roomId;
            const reserved = !!s.reserved && !wasMine && !net.status.is_local;
            // Seat 0 is the host's own: a guest never takes it.
            const host = s.seat === 0 && !net.status.is_local && (s.connected || !!s.reserved);
            return (
              <button
                key={s.seat}
                className={`join-seat ${s.connected ? 'taken' : ''} ${wasMine ? 'mine' : ''}`}
                style={{ ['--pc' as string]: factionColor(colored[i]) }}
                disabled={!!busy || host || reserved || (!!net.locked && !codeText && !wasMine)}
                onClick={() => join(s.seat, s.name)}
              >
                <div className="join-top">
                  <FactionMark faction={s.faction} size={64} />
                  <div>
                    <div className="join-name">{s.name}</div>
                    <div className="faction-motto">{info.name} · {info.motto}</div>
                  </div>
                </div>
                <div className="join-rule">
                  Strength: {FACTION_SHEETS[s.faction].pairs[0].strength}
                  {FACTION_SHEETS[s.faction].pairs[0].price ? ` Price: ${FACTION_SHEETS[s.faction].pairs[0].price}` : ''} Goal: {info.goal}
                </div>
                <div className="join-status">
                  <span className={`conn-dot ${s.connected ? 'on' : ''}`} />
                  {host ? 'Host' : s.connected ? 'In use: someone is playing this seat right now' : reserved ? 'Reserved' : wasMine ? 'Your seat' : 'Open'}
                </div>
                {!host && <div className="join-cta">{reserved ? 'Ask the host to open this seat' : s.connected ? 'Take over' : `Rejoin as ${s.name}`}</div>}
              </button>
            );
          })}
        </div>
        <ReclaimSeatButton />
      </div>
    </div>
  );
}
