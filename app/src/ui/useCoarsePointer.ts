import { useEffect, useState } from 'react';

export function useCoarsePointer() {
  const [coarse, setCoarse] = useState(() => globalThis.matchMedia?.('(pointer: coarse)').matches ?? false);
  useEffect(() => {
    const media = window.matchMedia('(pointer: coarse)');
    const update = () => setCoarse(media.matches);
    update();
    media.addEventListener?.('change', update);
    return () => media.removeEventListener?.('change', update);
  }, []);
  return coarse;
}
