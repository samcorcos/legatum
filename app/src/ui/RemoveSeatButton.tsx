import { useEffect, useRef, useState } from 'react';

/**
 * The host takes a remote player out of a seat. One click asks inline, with focus on the safe choice
 * (Keep); Esc keeps the player. The seat stays closed until the host opens it.
 */
export function RemoveSeatButton({ name, onRemove, disabled = false }: { name: string; onRemove: () => void; disabled?: boolean }) {
  const [asking, setAsking] = useState(false);
  const keep = useRef<HTMLButtonElement>(null);
  const confirm = useRef<HTMLSpanElement>(null);
  useEffect(() => {
    if (!asking) return;
    keep.current?.focus({ preventScroll: true });
    // The whole question must be in view: a destructive button half under a pinned bar is a trap.
    confirm.current?.scrollIntoView?.({ block: 'nearest' });
  }, [asking]);
  if (!asking) {
    return (
      <button className="btn danger remove-player" disabled={disabled} onClick={() => setAsking(true)} aria-label={`Remove ${name}`}>
        Remove
      </button>
    );
  }
  return (
    <span
      ref={confirm}
      className="remove-confirm"
      role="alertdialog"
      aria-label={`Remove ${name}?`}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          e.stopPropagation();
          e.preventDefault();
          setAsking(false);
        }
      }}
    >
      <span className="remove-ask">Remove {name}?</span>
      <button ref={keep} className="btn" onClick={() => setAsking(false)}>
        Keep
      </button>
      <button
        className="btn danger confirm"
        onClick={() => {
          setAsking(false);
          onRemove();
        }}
      >
        Remove
      </button>
    </span>
  );
}
