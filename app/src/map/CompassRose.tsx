// The map's compass rose uses Atlas ink on paper. An HTML overlay so it can
// fade when panels or map labels leave no clear placement; MapCanvas positions it each frame.
import { forwardRef } from 'react';

const TICKS = Array.from({ length: 8 }, (_, k) => {
  const a = (k * Math.PI) / 4 + Math.PI / 8;
  return `M${(Math.cos(a) * 24).toFixed(2)} ${(Math.sin(a) * 24).toFixed(2)}L${(Math.cos(a) * 28).toFixed(2)} ${(Math.sin(a) * 28).toFixed(2)}`;
}).join('');

export const CompassRose = forwardRef<HTMLDivElement>(function CompassRose(_, ref) {
  return (
    <div ref={ref} className="map-compass" data-overlay-slot="map-compass" aria-hidden>
      <svg viewBox="-40 -40 80 80" width="80" height="80">
        <circle r="28" className="c-ring" />
        <circle r="24" className="c-ring2" />
        <path d={TICKS} className="c-tick" />
        <path d="M0 -34L4 -4L0 0L-4 -4Z" className="c-ink" />
        <path d="M0 -34L-4 -4L0 0Z" className="c-fill" />
        <path d="M0 34L4 4L0 0L-4 4Z" className="c-fill c-edge" />
        <path d="M-30 0L-4 3L0 0L-4 -3Z" className="c-fill c-edge" />
        <path d="M30 0L4 3L0 0L4 -3Z" className="c-fill c-edge" />
        <text y="-37" textAnchor="middle" className="c-n">
          N
        </text>
      </svg>
    </div>
  );
});
