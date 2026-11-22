import { useEffect, useState } from 'react';

export function usePhoneLayout() {
  const [phone, setPhone] = useState(() => window.matchMedia('(max-width: 699px)').matches);
  useEffect(() => {
    const media = window.matchMedia('(max-width: 699px)');
    const update = () => setPhone(media.matches);
    update();
    media.addEventListener?.('change', update);
    return () => media.removeEventListener?.('change', update);
  }, []);
  return phone;
}
