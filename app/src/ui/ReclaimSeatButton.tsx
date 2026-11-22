import { useGame } from '../store';

/** A rejected credential never grants automatic permission to reassign a reservation. */
export function ReclaimSeatButton() {
  const allowed = useGame((s) => s.net?.status.is_local && s.net.credentialRejected);
  const busy = useGame((s) => s.busy);
  const reclaim = useGame((s) => s.reclaimOnlineSeat);
  if (!allowed) return null;
  return <button className="btn" disabled={!!busy} onClick={reclaim}>Reclaim my seat</button>;
}
