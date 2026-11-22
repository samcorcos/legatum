import { useEffect } from 'react';
import { useGame } from '../store';
import { rememberedSeat } from '../remote';
import { FACTION_INFO } from '../data';
import { FACTION_SHEETS } from '../factionSheets';
import { colorFactions, factionColor } from '../theme';
import { FactionMark } from '../ui/bits';
import { HexBackdrop } from './HexBackdrop';
import { ReclaimSeatButton } from '../ui/ReclaimSeatButton';

/** An online game: pick which human seat this browser plays. */
export function JoinScreen() {
  const net = useGame((s) => s.net);
  const exitToTitle = useGame((s) => s.exitToTitle);
  const joinSeat = useGame((s) => s.joinSeat);
  const refresh = useGame((s) => s.refreshStatus);
  const busy = useGame((s) => s.busy);
  const error = useGame((s) => s.error);
  useEffect(() => {
    refresh();
    const t = setInterval(() => refresh(), 2500);
    return () => clearInterval(t);
  }, [refresh]);
  if (!net) return null;
  const mine = rememberedSeat();
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
        {error && <div className="toast bad inline" role="alert">{error}</div>}
        <div className="join-seats">
          {seats.map((s, i) => {
            const info = FACTION_INFO[s.faction];
            const wasMine = mine?.seat === s.seat && !!mine.token && !mine.roomId;
            const reserved = !!s.reserved && !wasMine && !net.status.is_local;
            return (
              <button
                key={s.seat}
                className={`join-seat ${s.connected ? 'taken' : ''} ${wasMine ? 'mine' : ''}`}
                style={{ ['--pc' as string]: factionColor(colored[i]) }}
                disabled={!!busy || reserved}
                onClick={() => joinSeat(s.seat, s.name)}
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
                  {s.connected ? 'In use: someone is playing this seat right now' : reserved ? 'Reserved' : wasMine ? 'Your seat in this tab' : 'Open'}
                </div>
                <div className="join-cta">{reserved ? 'Ask the host to open this seat' : s.connected ? 'Take over' : `Rejoin as ${s.name}`}</div>
              </button>
            );
          })}
        </div>
        <ReclaimSeatButton />
      </div>
    </div>
  );
}
