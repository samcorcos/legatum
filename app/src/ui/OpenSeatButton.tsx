/** The host must deliberately invalidate a disconnected player's reservation. */
export function OpenSeatButton({ name, onOpen, disabled = false }: { name: string; onOpen: () => void; disabled?: boolean }) {
  return (
    <button
      className="btn quiet open-seat"
      disabled={disabled}
      onClick={() => {
        if (window.confirm(`Let another device take over ${name}'s empire?`)) onOpen();
      }}
    >
      Open seat
    </button>
  );
}
