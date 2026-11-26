// The era cinematics (`assets/cinematics/<era>.mp4`): a short film that plays full screen before the era takeover's
// card. Unlike the paintings there is no fallback: an era without its own film goes straight to the card.
import { CITY_ERAS } from '../map/cityArt';

const urls = import.meta.glob<string>('../assets/cinematics/*.mp4', { eager: true, query: '?url', import: 'default' });
const byEra = new Map<string, string>();
for (const [path, url] of Object.entries(urls)) byEra.set(path.replace(/^.*\/cinematics\//, '').replace(/\.mp4$/, ''), url);

/** The film for a 0-based era, or null when that era has none yet. */
export function eraCinematic(era: number): string | null {
  const name = CITY_ERAS[Math.round(era)];
  return (name && byEra.get(name)) || null;
}

// A film plays once per era per session: the report's "Show score check" reopens the takeover for its card, not the film.
const played = new Set<number>();
export const eraFilmPlayed = (era: number) => played.has(Math.round(era));
export const markEraFilmPlayed = (era: number) => { played.add(Math.round(era)); };
/** Tests only: forget which films have played. */
export const resetEraFilms = () => played.clear();
