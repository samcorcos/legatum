import { type ReactNode } from 'react';
import { useGame } from '../../store';
import { factionColor } from '../../theme';
import { FactionPicker } from '../../ui/FactionPicker';
import { OpenSeatButton } from '../../ui/OpenSeatButton';
import { RemoveSeatButton } from '../../ui/RemoveSeatButton';
import { Icon } from '../../ui/bits';
import { MAX_PLAYERS } from '../../gameSetup';
import { activeCount, addSlot, slotColorFactions, removeSlot, setSlot, slotKinds, type FactionChoice, type Lobby, type LobbyMode, type Slot, type SlotKind } from '../../lobby';
import type { StagingState } from '../../staging';

const KIND_LABEL: Record<SlotKind, string> = { you: 'You', human: 'Human', ai: 'AI', closed: 'Closed' };

/** The staging room as seen from one browser. */
export interface RoomView {
  state: StagingState;
  /** A network player's view (read-only settings) rather than the host's. */
  guest: boolean;
  /** Guest: the seat this browser holds. */
  mySeat?: number | null;
  /** Guest: what to show in an open seat's name cell (a Take seat button, or your own controls). */
  renderOpen?: (seat: number) => ReactNode;
  /** Guest: pick the faction of the seat this browser holds, in that seat's own row. */
  onMyFaction?: (faction: FactionChoice) => void;
}

/** The seats table. Without `onChange` it's read-only. */
export function SeatsTable({
  lobby,
  shown = lobby,
  onChange,
  room,
}: {
  lobby: Lobby;
  /** The seats as displayed, when they differ from the edited lobby (network players' own picks). */
  shown?: Lobby;
  onChange?: (l: Lobby) => void;
  room?: RoomView;
}) {
  const colored = slotColorFactions(lobby);
  return (
    <section className="lobby-card lobby-seats">
      <div className="lobby-card-head">
        <h3>Empires</h3>
        <span className="muted">
          {activeCount(lobby)} of {MAX_PLAYERS} seats
        </span>
      </div>
      <div className="seat-table" role="table" aria-label="Seats">
        <div className="seat-row head" role="row">
          <span />
          <span className="lbl">Seat</span>
          <span className="lbl">{room ? 'Player' : 'Name'}</span>
          <span className="lbl">Faction</span>
          <span className="lbl">Team</span>
          <span />
        </div>
        {lobby.slots.map((s, i) => (
          <SeatRow
            key={i}
            index={i}
            slot={shown.slots[i] ?? s}
            mode={lobby.mode}
            color={colored[i] === null ? '#3a3f47' : factionColor(colored[i])}
            canRemove={!!onChange && i > 0 && lobby.slots.length > 2}
            onChange={onChange ? (patch) => onChange(setSlot(lobby, i, patch)) : undefined}
            onRemove={() => onChange?.(removeSlot(lobby, i))}
            room={room}
          />
        ))}
      </div>
      {onChange && lobby.slots.length < MAX_PLAYERS && (
        <button className="btn add-seat" onClick={() => onChange(addSlot(lobby))}>
          + Add an AI empire
        </button>
      )}
    </section>
  );
}

function Occupancy({ room, seat }: { room: RoomView; seat: number }) {
  const local = useGame((s) => s.net?.status.is_local);
  const mySeat = useGame((s) => s.stagingSeat);
  const busy = useGame((s) => s.busy);
  const releaseSeat = useGame((s) => s.releaseStagingSeat);
  const removePlayer = useGame((s) => s.removeStagingPlayer);
  const o = room.state.occupants[seat];
  if (!o) return <span className="occupant empty">Open · waiting for a player…</span>;
  const status = !o.connected ? 'disconnected' : o.is_host ? 'host' : o.ready ? 'Ready' : 'not ready';
  return (
    <span className={`occupant ${o.ready || o.is_host ? 'ready' : ''}`}>
      <span className="occupant-name">{o.name}</span>
      <span className="occupant-status">
        {o.ready && !o.is_host && <><Icon name="ui-check" size={12} className="ui-mark" /> </>}
        {status}
      </span>
      {!room.guest && local && seat !== mySeat && !o.is_host && !o.connected && (
        <OpenSeatButton name={o.name} disabled={!!busy} onOpen={() => releaseSeat(seat)} />
      )}
      {!room.guest && local && seat !== mySeat && !o.is_host && o.connected && (
        <RemoveSeatButton name={o.name} disabled={!!busy} onRemove={() => removePlayer(seat)} />
      )}
    </span>
  );
}

function SeatRow({
  index,
  slot,
  mode,
  color,
  canRemove,
  onChange,
  onRemove,
  room,
}: {
  index: number;
  slot: Slot;
  mode: LobbyMode;
  color: string;
  canRemove: boolean;
  onChange?: (patch: Partial<Slot>) => void;
  onRemove: () => void;
  room?: RoomView;
}) {
  const closed = slot.kind === 'closed';
  const kinds = slotKinds(mode);
  const kindLabel = (k: SlotKind) => (mode === 'host' && k === 'human' ? 'Online player' : KIND_LABEL[k]);
  const readOnly = !onChange;
  const networkSeat = !!room && (slot.kind === 'human' || slot.kind === 'you');
  const youLabel = room?.guest ? 'Host' : mode === 'host' ? 'You · host' : 'You';
  const mine = room?.guest && room.mySeat === index;

  let nameCell: ReactNode = null;
  if (closed) nameCell = null;
  else if (room?.guest && slot.kind === 'human' && room.renderOpen) nameCell = room.renderOpen(index);
  else if (networkSeat && slot.kind === 'human') nameCell = <Occupancy room={room!} seat={index} />;
  else if (readOnly) nameCell = <span className="seat-text">{slot.name}</span>;
  else
    nameCell = (
      <input
        className="lobby-input"
        value={slot.name}
        maxLength={24}
        aria-label={`Seat ${index + 1} name`}
        placeholder="Name"
        onChange={(e) => onChange?.({ name: e.target.value })}
      />
    );

  return (
    <div className={`seat-row ${closed ? 'closed' : ''} ${mine ? 'mine' : ''}`} role="row" style={{ ['--pc' as string]: color }}>
      <span className="seat-swatch" aria-hidden />
      <span>
        {mine ? (
          <span className="seat-you">You</span>
        ) : slot.kind === 'you' ? (
          <span className="seat-you">{youLabel}</span>
        ) : readOnly ? (
          <span className="seat-text muted">{kindLabel(slot.kind)}</span>
        ) : (
          <select
            className="lobby-select"
            value={slot.kind}
            aria-label={`Seat ${index + 1} type`}
            onChange={(e) => onChange?.({ kind: e.target.value as SlotKind })}
          >
            {kinds.map((k) => (
              <option key={k} value={k}>
                {kindLabel(k)}
              </option>
            ))}
          </select>
        )}
      </span>
      <span>{nameCell}</span>
      <span>
        {/* Every player picks their faction in their own seat's row; everyone else sees it there. */}
        {!closed && (mine && room?.onMyFaction ? (
          <FactionPicker value={slot.faction} label="Your faction" onChange={room.onMyFaction} />
        ) : (
          <FactionPicker
            value={slot.faction}
            label={`Seat ${index + 1} faction`}
            // Online players pick their own faction.
            disabled={readOnly || (networkSeat && slot.kind === 'human')}
            onChange={(faction) => onChange?.({ faction })}
          />
        ))}
      </span>
      <span>
        {!closed &&
          (readOnly ? (
            <span className="seat-text">Team {slot.team + 1}</span>
          ) : (
            <select
              className="lobby-select"
              value={slot.team}
              aria-label={`Seat ${index + 1} team`}
              onChange={(e) => onChange?.({ team: Number(e.target.value) })}
            >
              {Array.from({ length: MAX_PLAYERS }, (_, t) => (
                <option key={t} value={t}>
                  Team {t + 1}
                </option>
              ))}
            </select>
          ))}
      </span>
      <span>
        {canRemove && (
          <button className="x seat-remove" onClick={onRemove} title="Remove this seat" aria-label={`Remove seat ${index + 1}`}>
            <Icon name="ui-cross" size={11} />
          </button>
        )}
      </span>
    </div>
  );
}
