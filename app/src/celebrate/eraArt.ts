// The era takeover's full-bleed paintings (`assets/eras/<era>-1280.jpg` and `-1920.jpg`, derived from the key-art
// masters in docs/design/era-art). A missing era falls back to the nearest finished one, never to a blank screen.
import { CITY_ERAS } from '../map/cityArt';

const urls = import.meta.glob<string>('../assets/eras/*-{1280,1920}.jpg', { eager: true, query: '?url', import: 'default' });
const byKey = new Map<string, string>();
for (const [path, url] of Object.entries(urls)) byKey.set(path.replace(/^.*\/eras\//, '').replace(/\.jpg$/, ''), url);

/** The painting for a 0-based era: a 1280 px and a 1920 px source for `srcset`. */
export function eraArt(era: number): { era: string; small: string; large: string } | null {
  const e = Math.min(CITY_ERAS.length - 1, Math.max(0, Math.round(era)));
  for (let d = 0; d < CITY_ERAS.length; d++) {
    for (const j of [e - d, e + d]) {
      const name = CITY_ERAS[j];
      const small = name && byKey.get(`${name}-1280`);
      const large = name && byKey.get(`${name}-1920`);
      if (small && large) return { era: name, small, large };
    }
  }
  return null;
}
