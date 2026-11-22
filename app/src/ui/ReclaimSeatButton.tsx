import { useGame } from '../store';

/** A rejected credential never grants automatic permission to reassign a reservation. */
export function ReclaimSeatButton() {
  const allowed = useGame((s) => s.lan?.status.is_local && s.lan.credentialRejected);
  const busy = useGame((s) => s.busy);
  const reclaim = useGame((s) => s.reclaimLanSeat);
  if (!allowed) return null;
  return <button className="btn" disabled={!!busy} onClick={reclaim}>Reclaim my seat</button>;
}
