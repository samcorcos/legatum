// The title screen's painted backdrop: the key art, covering the screen, drifting slowly (a long
// Ken Burns zoom) with a few pixels of pointer parallax, under a parchment wash
// that keeps the wordmark and menu legible. Transforms only, so it stays
// on the compositor; both motions stop under prefers-reduced-motion.
import { useEffect, useRef } from 'react';
import keyArt from '../assets/art/title-keyart-c-road-at-dawn-1600.jpg';

export function TitleArt() {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = ref.current;
    if (!el || window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return;
    let raf = 0;
    let tx = 0;
    let ty = 0;
    const move = (e: PointerEvent) => {
      tx = (e.clientX / window.innerWidth - 0.5) * -2;
      ty = (e.clientY / window.innerHeight - 0.5) * -2;
      if (raf) return;
      raf = requestAnimationFrame(() => {
        raf = 0;
        el.style.setProperty('--px', tx.toFixed(3));
        el.style.setProperty('--py', ty.toFixed(3));
      });
    };
    window.addEventListener('pointermove', move);
    return () => {
      window.removeEventListener('pointermove', move);
      cancelAnimationFrame(raf);
    };
  }, []);
  return (
    <div className="title-art" ref={ref} aria-hidden>
      <div className="title-art-drift">
        <img src={keyArt} alt="" draggable={false} decoding="async" />
      </div>
      <div className="title-art-wash" />
    </div>
  );
}
