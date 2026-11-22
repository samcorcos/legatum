import { WorldBackdrop } from '../map/WorldBackdrop';

/** The painted world under a strong veil, behind the setup, lobby and join screens. */
export function HexBackdrop() {
  return (
    <>
      <WorldBackdrop />
      <div className="backdrop-veil" />
    </>
  );
}
